#!/usr/bin/env -S godot --script
# LLMBridge — Puente bidireccional runtime LLM ↔ Godot (Autoload NexusLLMBridge)
# Permite al backend Rust/Python (y por tanto al LLM) controlar el juego EN TIEMPO REAL:
# leer el árbol de escena, spawnear bestias, mover al jugador, inyectar daño, etc.
#
# Transportes expuestos (todos sobre el MISMO puerto):
#   - HTTP  : GET /health, GET /scene/tree, GET /player, POST /command
#   - WebSocket: ws://127.0.0.1:8081/nexus   (protocolo JSON-RPC-ish)
#
# Formato de comando (HTTP POST /command o mensaje WS):
#   { "id": "opcional", "cmd": "SPAWN_BEAST", "args": { "species": "lobo", "x": 5.0, "z": -3.0 } }

extends Node

# ─── CONFIG ─────────────────────────────────────────────────────
const PORT = 8081
const WS_PATH = "/nexus"
const AUTH_TOKEN = "nexus-llm-bridge-2026"  # Cambia en producción
const HTTP_VERSION_STR = "HTTP/1.1"

# Registro de bestias disponibles (especies → ruta del recurso BeastData)
const BEAST_REGISTRY: Dictionary = {
	"lobo":   "res://scripts/beasts/wolf_data.tres",
	"boar":   "res://scripts/beasts/boar_data.tres",
	"jabali": "res://scripts/beasts/boar_data.tres",
	"spider": "res://scripts/beasts/spider_data.tres",
	"arana":  "res://scripts/beasts/spider_data.tres",
	"bat":    "res://scripts/beasts/bat_data.tres",
	"murcielago": "res://scripts/beasts/bat_data.tres",
	"golem":  "res://scripts/beasts/golem_data.tres",
}

# ─── ESTADO ─────────────────────────────────────────────────────
var _server: TCPServer = null
var _clients: Array[StreamPeerTCP] = []
var _ws_peers: Array[WebSocketPeer] = []
var _pending_responses: Dictionary = {}  # id → String (respuesta JSON pendiente)
var _next_id: int = 0

# ─── CICLO DE VIDA ──────────────────────────────────────────────
func _ready() -> void:
	_server = TCPServer.new()
	var err := _server.listen(PORT, "127.0.0.1")
	if err == OK:
		print("[LLM-BRIDGE] Puente activo en 127.0.0.1:%d (HTTP + WebSocket)" % PORT)
	else:
		printerr("[LLM-BRIDGE] ERROR iniciando en puerto %d: %s" % [PORT, err])
	set_process(true)

func _exit_tree() -> void:
	for c in _clients:
		c.disconnect_from_host()
	_clients.clear()
	for w in _ws_peers:
		w.close()
	_ws_peers.clear()
	if _server:
		_server.stop()
		_server = null

func _process(_delta: float) -> void:
	_process_new_connections()
	_process_http_clients()
	_process_ws_clients()

# ─── CONEXIONES ─────────────────────────────────────────────────
func _process_new_connections() -> void:
	while _server != null and _server.is_connection_available():
		var conn: StreamPeerTCP = _server.take_connection()
		# Auto-detección: si la primera línea trae el upgrade, es WS; si no, HTTP clásico.
		conn.poll()
		if conn.get_status() != StreamPeerTCP.STATUS_CONNECTED:
			continue
		# Leer primer chunk para detectar GET /nexus con Upgrade.
		var peek := conn.get_available_bytes()
		if peek <= 0:
			_clients.append(conn)
			continue
		var header := conn.get_utf8_string(peek)
		if header.find("Upgrade: websocket") != -1 and header.find(WS_PATH) != -1:
			_handle_ws_upgrade(conn, header)
			continue
		# No es WebSocket: procesar como HTTP con los bytes ya leídos.
		_handle_http_request(conn, header)

func _process_http_clients() -> void:
	var to_remove: Array[StreamPeerTCP] = []
	for c in _clients:
		c.poll()
		match c.get_status():
			StreamPeerTCP.STATUS_CONNECTED:
				if c.get_available_bytes() > 0:
					var data := c.get_utf8_string(c.get_available_bytes())
					_handle_http_request(c, data)
					to_remove.append(c)  # respuesta única por petición HTTP
			StreamPeerTCP.STATUS_ERROR, StreamPeerTCP.STATUS_NONE:
				to_remove.append(c)
	for c in to_remove:
		_clients.erase(c)

func _process_ws_clients() -> void:
	for w in _ws_peers:
		if w.get_ready_state() == WebSocketPeer.STATE_OPEN:
			w.poll()
			if w.get_ready_state() != WebSocketPeer.STATE_OPEN:
				continue
			while w.get_available_packet_count() > 0:
				var msg: PackedByteArray = w.get_packet()
				if not msg.is_empty():
					_handle_command_string(w, msg.get_string_from_utf8(), "ws")
		elif w.get_ready_state() == WebSocketPeer.STATE_CLOSED:
			_ws_peers.erase(w)

# ─── HTTP ───────────────────────────────────────────────────────
func _handle_http_request(conn: StreamPeerTCP, raw: String) -> void:
	var lines := raw.split("\r\n")
	if lines.is_empty():
		_send_http(conn, 400, "Bad Request")
		return
	var request_line := lines[0].split(" ")
	if request_line.size() < 2:
		_send_http(conn, 400, "Bad Request")
		return
	var method := request_line[0]
	var path := request_line[1]

	# Leer body (si hay Content-Length)
	var body := ""
	for i in range(1, lines.size()):
		if lines[i].begins_with("Content-Length:"):
			var clen := int(lines[i].get_slice(":", 1).strip_edges())
			# el body suele quedar tras la cabecera en `raw`; aproximamos tomando el resto
			var marker := raw.find("\r\n\r\n")
			if marker != -1:
				var raw_body := raw.substr(marker + 4)
				if raw_body.length() >= clen:
					body = raw_body.substr(0, clen)
			break

	match method:
		"GET":
			match path:
				"/health":
					_send_json(conn, {"status": "ok", "bridge": "nexus-llm-bridge", "version": "1.0.0", "port": PORT})
				"/scene/tree":
					_send_json(conn, {"scene_tree": _serialize_scene_tree(get_tree().current_scene)})
				"/player":
					_send_json(conn, {"player": _get_player_info()})
				_:
					_send_http(conn, 404, "Not Found")
		"POST":
			if path == "/command":
				var data: Variant = JSON.parse_string(body)
				if data is Dictionary and data.has("cmd"):
					var resp := _dispatch(data.cmd, data.get("args", {}))
					_send_json(conn, {"ok": true, "id": data.get("id", ""), "result": resp})
				else:
					_send_json(conn, {"ok": false, "error": "Formato: {cmd, args?}"}, 400)
			else:
				_send_http(conn, 404, "Not Found")
		_:
			_send_http(conn, 405, "Method Not Allowed")
	# Cerrar la conexión HTTP tras responder (Connection: close).
	conn.disconnect_from_host()

# ─── WEB SOCKET ─────────────────────────────────────────────────
func _handle_ws_upgrade(conn: StreamPeerTCP, header: String) -> void:
	var key := ""
	for line in header.split("\r\n"):
		if line.begins_with("Sec-WebSocket-Key:"):
			key = line.get_slice(":", 1).strip_edges()
			break
	if key.is_empty():
		_send_http(conn, 400, "Missing Sec-WebSocket-Key")
		return
	var accept := _ws_accept_key(key)
	var resp := "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: %s\r\n\r\n" % accept
	conn.put_data(resp.to_utf8_buffer())
	# Crear WebSocketPeer adosado a esta conexión raw
	var peer := WebSocketPeer.new()
	peer.accept_stream(conn)
	_ws_peers.append(peer)

func _ws_accept_key(key: String) -> String:
	var concat := (key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").to_utf8_buffer()
	var hasher := HashingContext.new()
	hasher.start(HashingContext.HASH_SHA1)
	hasher.update(concat)
	var digest := hasher.finish()
	return Marshalls.raw_to_base64(digest)

func _handle_command_string(from, raw: String, source: String) -> void:
	var data: Variant = JSON.parse_string(raw)
	if data is Dictionary and data.has("cmd"):
		var resp := _dispatch(data.cmd, data.get("args", {}))
		var payload := JSON.stringify({"ok": true, "id": data.get("id", ""), "result": resp})
		if from is WebSocketPeer:
			from.send_text(payload)

# ─── DESPACHO DE COMANDOS ───────────────────────────────────────
func _dispatch(cmd: String, args: Dictionary) -> Dictionary:
	match cmd:
		"PING":
			return {"pong": true}
		"GET_PLAYER":
			return _get_player_info()
		"GET_SCENE_TREE":
			return {"tree": _serialize_scene_tree(get_tree().current_scene)}
		"LIST_BEASTS":
			return {"species": BEAST_REGISTRY.keys()}
		"SPAWN_BEAST":
			return _cmd_spawn_beast(args)
		"KILL_BEASTS":
			return _cmd_kill_beasts()
		"DAMAGE_PLAYER":
			return _cmd_damage_player(args)
		"HEAL_PLAYER":
			return _cmd_heal_player(args)
		"MOVE_PLAYER":
			return _cmd_move_player(args)
		"GET_TIME":
			return {"ticks": Time.get_ticks_msec(), "delta": _delta_approx()}
		_:
			return {"error": "Comando desconocido: %s" % cmd}

func _delta_approx() -> float:
	return 0.016

# ─── COMANDOS CONCRETOS ─────────────────────────────────────────
func _cmd_spawn_beast(args: Dictionary) -> Dictionary:
	var species := str(args.get("species", "lobo")).to_lower()
	if not BEAST_REGISTRY.has(species):
		return {"error": "Especie desconocida. Disponibles: %s" % BEAST_REGISTRY.keys()}
	var res_path: String = BEAST_REGISTRY[species]
	var beast_data: BeastData = load(res_path)
	if beast_data == null:
		return {"error": "No se pudo cargar BeastData: %s" % res_path}

	var pos := Vector3.ZERO
	if args.has("x") and args.has("z"):
		pos = Vector3(float(args.x), 0.0, float(args.z))
	else:
		var player := _get_player_node()
		if player != null:
			pos = player.global_position + Vector3(5.0, 0.0, 5.0)

	var beast_scene := load("res://scenes/beasts/beast_base.tscn")
	if beast_scene == null:
		return {"error": "No se pudo cargar beast_base.tscn"}
	var beast: Beast = beast_scene.instantiate()
	beast.global_position = pos
	beast.beast_data = beast_data
	beast.name = "%s_llm_%d" % [species, _next_id]
	_next_id += 1
	var world := get_tree().current_scene
	world.add_child(beast)
	return {"spawned": beast.name, "position": [pos.x, pos.y, pos.z]}

func _cmd_kill_beasts() -> Dictionary:
	var count := 0
	for node in get_tree().get_nodes_in_group("enemies"):
		if node is Beast and is_instance_valid(node):
			node.queue_free()
			count += 1
	return {"killed": count}

func _cmd_damage_player(args: Dictionary) -> Dictionary:
	var player := _get_player_node()
	if player == null:
		return {"error": "Jugador no encontrado"}
	var amount := float(args.get("amount", 10.0))
	if player.has_method("take_damage"):
		player.take_damage(amount, null)
	return {"damage": amount, "method": "take_damage"}

func _cmd_heal_player(args: Dictionary) -> Dictionary:
	var player := _get_player_node()
	if player == null:
		return {"error": "Jugador no encontrado"}
	var health := player.get_node_or_null("HealthComponent")
	if health == null:
		return {"error": "Sin HealthComponent"}
	var amount := float(args.get("amount", 999.0))
	health.heal(amount)
	return {"healed": amount}

func _cmd_move_player(args: Dictionary) -> Dictionary:
	var player := _get_player_node()
	if player == null:
		return {"error": "Jugador no encontrado"}
	var x := float(args.get("x", 0.0))
	var z := float(args.get("z", 0.0))
	if args.has("x") and args.has("z"):
		player.global_position = Vector3(x, player.global_position.y, z)
	return {"moved": [player.global_position.x, player.global_position.y, player.global_position.z]}

# ─── HELPERS ────────────────────────────────────────────────────
func _get_player_node() -> Node3D:
	return get_tree().get_first_node_in_group("player") as Node3D

func _get_player_info() -> Dictionary:
	var player := _get_player_node()
	if player == null:
		return {"present": false}
	var info := {
		"present": true,
		"name": player.name,
		"position": [player.global_position.x, player.global_position.y, player.global_position.z],
	}
	var health := player.get_node_or_null("HealthComponent")
	if health != null:
		info["health"] = health.current_health
		info["max_health"] = health.max_health
		info["health_pct"] = health.get_health_percentage()
	return info

func _serialize_scene_tree(node: Node, depth: int = 0) -> Dictionary:
	if node == null or depth > 12:
		return {}
	var d := {
		"name": node.name,
		"type": node.get_class(),
		"children": [],
	}
	if node is Node3D:
		d["position"] = [node.global_position.x, node.global_position.y, node.global_position.z]
	for child in node.get_children():
		d["children"].append(_serialize_scene_tree(child, depth + 1))
	return d

func _send_json(conn: StreamPeerTCP, obj: Dictionary, status: int = 200) -> void:
	var body := JSON.stringify(obj)
	var headers := "%s %d %s\r\nContent-Type: application/json\r\nContent-Length: %d\r\nConnection: close\r\n\r\n" % [HTTP_VERSION_STR, status, _status_text(status), body.length()]
	conn.put_data(headers.to_utf8_buffer())
	conn.put_data(body.to_utf8_buffer())

func _send_http(conn: StreamPeerTCP, status: int, text: String) -> void:
	var body := text
	var headers := "%s %d %s\r\nContent-Type: text/plain\r\nContent-Length: %d\r\nConnection: close\r\n\r\n" % [HTTP_VERSION_STR, status, _status_text(status), body.length()]
	conn.put_data(headers.to_utf8_buffer())
	conn.put_data(body.to_utf8_buffer())

func _status_text(code: int) -> String:
	match code:
		200: return "OK"
		400: return "Bad Request"
		404: return "Not Found"
		405: return "Method Not Allowed"
		_: return "Unknown"
