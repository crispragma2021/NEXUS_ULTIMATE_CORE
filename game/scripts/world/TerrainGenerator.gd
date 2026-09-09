## TerrainGenerator.gd — Generador procedural de terreno por chunks.
## NEXUS Protocol · Fase 1 · Generación procedural
class_name TerrainGenerator
extends Node

## Genera un chunk completo (malla de terreno + bioma) y lo devuelve.
## @param chunk_id Coordenadas del chunk en unidades de chunk.
## @param parent Contenedor donde añadir el chunk (opcional).
## @returns Nodo Chunk con la malla de terreno generada.
func generate_chunk(chunk_id: Vector2i, parent: Node3D = null) -> Node3D:
	var chunk := Node3D.new()
	chunk.name = "Chunk_%d_%d" % [chunk_id.x, chunk_id.y]
	if parent != null:
		parent.add_child(chunk)

	var origin := NexusWorld.chunk_to_world_origin(chunk_id)
	chunk.position = origin

	_build_terrain_mesh(chunk, chunk_id)
	return chunk

## Construye la malla de terreno del chunk usando altura por ruido.
func _build_terrain_mesh(chunk: Node3D, chunk_id: Vector2i) -> void:
	var cs := NexusConfig.CHUNK_SIZE
	var noise := FastNoiseLite.new()
	noise.seed = NexusConfig.seed_base + (chunk_id.x * 73856093) + (chunk_id.y * 19349663)
	noise.noise_type = FastNoiseLite.TYPE_PERLIN
	noise.frequency = NexusConfig.NOISE_FREQUENCY
	noise.fractal_octaves = NexusConfig.NOISE_OCTAVES
	noise.fractal_lacunarity = NexusConfig.NOISE_LACUNARITY
	noise.fractal_gain = NexusConfig.NOISE_GAIN

	var size_x := cs.x
	var size_z := cs.y
	var total_vertices := (size_x + 1) * (size_z + 1)

	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)

	var heights := {}
	var min_h := INF
	var max_h := -INF

	# Calcular alturas para todos los vértices (incluye borde compartido).
	for z in range(size_z + 1):
		for x in range(size_x + 1):
			var wx := chunk.position.x + float(x)
			var wz := chunk.position.z + float(z)
			# Coordenadas de ruido globales para continuidad entre chunks.
			var n := noise.get_noise_2d(wx, wz)
			var h := _height_from_noise(n)
			heights[Vector2i(x, z)] = h
			min_h = min(min_h, h)
			max_h = max(max_h, h)

	# Colores por altura para estética low-poly (gradiente de bioma).
	var color_low := Color(0.36, 0.55, 0.32)   # pasto
	var color_mid := Color(0.52, 0.62, 0.35)   # pasto alto / tierra
	var color_high := Color(0.60, 0.60, 0.58)  # roca

	# Construir vértices.
	for z in range(size_z + 1):
		for x in range(size_x + 1):
			var h: float = heights[Vector2i(x, z)]
			var t := 0.0
			if max_h > min_h:
				t = (h - min_h) / (max_h - min_h)
			var col := color_low.lerp(color_mid, t)
			if t > 0.6:
				col = color_mid.lerp(color_high, (t - 0.6) / 0.4)
			st.set_color(col)
			st.add_vertex(Vector3(float(x), h, float(z)))

	# Construir índices (2 triángulos por celda).
	for z in range(size_z):
		for x in range(size_x):
			var i0 := z * (size_x + 1) + x
			var i1 := i0 + 1
			var i2 := (z + 1) * (size_x + 1) + x
			var i3 := i2 + 1
			st.add_index(i0)
			st.add_index(i2)
			st.add_index(i1)
			st.add_index(i1)
			st.add_index(i2)
			st.add_index(i3)

	# Calcular normales suaves para un look low-poly limpio.
	st.generate_normals()
	var mesh := st.commit()

	var mesh_instance := MeshInstance3D.new()
	mesh_instance.mesh = mesh
	mesh_instance.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON
	# Material que usa los colores de vértice como textura (low-poly estético)
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_PER_PIXEL
	mat.albedo_color = Color.WHITE
	mesh_instance.material_override = mat
	chunk.add_child(mesh_instance)

	_add_collision(chunk, mesh)

## Devuelve una altura dentro del rango configurado a partir del valor de ruido (-1..1).
func _height_from_noise(n: float) -> float:
	var norm := (n + 1.0) * 0.5  # mapear a 0..1
	var h := lerpf(NexusConfig.TERRAIN_MIN_HEIGHT, NexusConfig.TERRAIN_MAX_HEIGHT, norm)
	# Escalonado sutil para estética low-poly (cajas discretas).
	return floorf(h * 2.0) / 2.0

## Añade un StaticBody3D con colisión a partir de la malla.
func _add_collision(chunk: Node3D, mesh: Mesh) -> void:
	var body := StaticBody3D.new()
	body.collision_layer = 1   # capa "terreno"
	body.collision_mask = 0
	var shape := CollisionShape3D.new()
	var concave := ConcavePolygonShape3D.new()
	concave.set_faces(mesh.get_faces() if mesh is ArrayMesh else PackedVector3Array())
	shape.shape = concave
	body.add_child(shape)
	chunk.add_child(body)
