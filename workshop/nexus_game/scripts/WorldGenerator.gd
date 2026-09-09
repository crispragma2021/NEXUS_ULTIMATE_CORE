extends Node3D

# WorldGenerator.gd - Generador Procedural de Chunks 3D en tiempo real
# Conectado al Servidor Rust de NEXUS vía WebSockets (ws://127.0.0.1:43212)

var _socket := WebSocketPeer.new()
const SERVER_URL := "ws://127.0.0.1:43212"

var noise := FastNoiseLite.new()
var chunks := {}

func _ready() -> void:
	print("🌐 [NEXUS GAME] Inicializando WorldGenerator...")
	print("🔌 Conectando a NEXUS Core Backend en: ", SERVER_URL)
	
	noise.noise_type = FastNoiseLite.TYPE_PERLIN
	noise.frequency = 0.05
	
	var err = _socket.connect_to_url(SERVER_URL)
	if err != OK:
		print("❌ Error al iniciar conexión WebSocket: ", err)

func _process(_delta: float) -> void:
	_socket.poll()
	var state = _socket.get_ready_state()
	
	if state == WebSocketPeer.STATE_OPEN:
		while _socket.get_available_packet_count() > 0:
			var packet = _socket.get_packet()
			var text = packet.get_string_from_utf8()
			_on_server_message(text)
	elif state == WebSocketPeer.STATE_CLOSED:
		var code = _socket.get_close_code()
		var reason = _socket.get_close_reason()
		# Reintento suave deshabilitado en log continuo

func _on_server_message(raw_msg: String) -> void:
	print("📥 [WEBSOCKET RECEIVED]: ", raw_msg)
	var json = JSON.parse_string(raw_msg)
	if json is Dictionary:
		var action = json.get("action", "")
		if action == "generate_procedural_chunk" or action == "generate_chunk":
			var cx = int(json.get("chunk_x", 0))
			var cz = int(json.get("chunk_z", 0))
			var seed_val = int(json.get("seed", 42))
			var biome = str(json.get("biome_type", "forest"))
			
			generate_chunk(cx, cz, seed_val, biome)
			
			# Confirmación enviada al servidor Rust
			var response = {
				"status": "chunk_ready",
				"chunk_x": cx,
				"chunk_z": cz,
				"biome": biome
			}
			_socket.send_text(JSON.stringify(response))

func generate_chunk(chunk_x: int, chunk_z: int, seed_val: int, biome: String) -> void:
	var chunk_key = str(chunk_x) + "_" + str(chunk_z)
	if chunks.has(chunk_key):
		print("⚠️ Chunk ", chunk_key, " ya fue generado.")
		return

	noise.seed = seed_val
	print("🌲 Generando Chunk (", chunk_x, ", ", chunk_z, ") Bioma: ", biome, " Semilla: ", seed_val)
	
	var plane_mesh := PlaneMesh.new()
	plane_mesh.size = Vector2(16, 16)
	plane_mesh.subdivide_width = 8
	plane_mesh.subdivide_depth = 8
	
	var surface_tool := SurfaceTool.new()
	surface_tool.create_from(plane_mesh, 0)
	var mdata := ArrayMesh.new()
	surface_tool.commit(mdata)

	var mesh_inst := MeshInstance3D.new()
	mesh_inst.mesh = mdata
	mesh_inst.position = Vector3(chunk_x * 16.0, 0.0, chunk_z * 16.0)

	# Material según el bioma
	var mat := StandardMaterial3D.new()
	match biome:
		"desert":
			mat.albedo_color = Color(0.9, 0.75, 0.4) # Arena
		"tundra":
			mat.albedo_color = Color(0.85, 0.9, 0.95) # Nieve
		_:
			mat.albedo_color = Color(0.2, 0.7, 0.3) # Bosque
			
	mat.roughness = 0.8
	mesh_inst.material_override = mat

	add_child(mesh_inst)
	chunks[chunk_key] = mesh_inst
	print("✅ Chunk ", chunk_key, " instanciado en 3D exitosamente.")
