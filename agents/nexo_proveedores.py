# Proveedores de modelos para NEXUS
# ORDEN_DEFAULT define el orden de selección de proveedores por defecto
# NEXUS_CASCADE_ORDER permite restringir la lista exacta de proveedores activos

ORDEN_DEFAULT = ["gemini", "deepseek", "openrouter", "groq", "github", "cerebras", "ollama"]

# Modelos disponibles por proveedor
MODELOS_PROVEEDOR = {
    "gemini": ["gemini-3.8-flash", "gemini-3.7-flash"],
    "deepseek": ["deepseek-chat"],
    "openrouter": ["meta-llama/llama-3.3-70b-instruct"],
    "groq": ["openai/gpt-oss-20b"],
    "github": ["gpt-4.1-mini"],
    "cerebras": [],
    "ollama": [],
}

# Mapeo inverso: proveedor -> lista de modelos
PROVEEDOR_MODELO = MODELOS_PROVEEDOR


def obtener_modelos(proveedor=None, orden=None):
    """
    Obtiene la lista de modelos para un proveedor o para el orden por defecto.

    Args:
        proveedor: Nombre del proveedor (ej. 'gemini'). Si None, usa el orden por defecto.
        orden: Lista personalizada de proveedores. Si None, usa ORDEN_DEFAULT o NEXUS_CASCADE_ORDER.

    Returns:
        list: Lista de nombres de modelo disponibles.

    Raises:
        RuntimeError: Si ORDEN_DEFAULT nombra proveedores no registrados.
    """
    # Determinar qué proveedores usar basándose en el parámetro 'orden'
    if orden is not None and len(orden) > 0:
        proveedores = orden
    else:
        # Usar NEXUS_CASCADE_ORDER si está definido para restringir la lista exacta
        cascade_order = None
        import os
        cascade_env = os.environ.get("NEXUS_CASCADE_ORDER")
        if cascade_env:
            cascade_order = [s.strip() for s in cascade_env.split(",") if s.strip()]
        if cascade_order is not None:
            proveedores = cascade_order
        else:
            proveedores = ORDEN_DEFAULT

    # Validar que todos los proveedores en la lista estén registrados
    for prov in proveedores:
        if prov not in MODELOS_PROVEEDOR:
            raise RuntimeError(
                f"Proveedor '{prov}' no registrado. "
                f"Proveedores disponibles: {list(MODELOS_PROVEEDOR.keys())}"
            )

    # Si se especificó un proveedor único, solo devolver sus modelos
    if proveedor is not None:
        if proveedor not in MODELOS_PROVEEDOR:
            raise RuntimeError(
                f"Proveedor '{proveedor}' no registrado. "
                f"Proveedores disponibles: {list(MODELOS_PROVEEDOR.keys())}"
            )
        return list(MODELOS_PROVEEDOR[proveedor])

    # Construir lista de modelos de todos los proveedores en la lista
    modelos = []
    for prov in proveedores:
        modelos.extend(MODELOS_PROVEEDOR[prov])

    return modelos


def obtener_proveedor_actual(orden=None):
    """Devuelve el siguiente proveedor en la secuencia de cascada."""
    proveedores = obtener_proveedores(orden=orden)
    if not proveedores:
        return None
    return proveedores[0]


def obtener_proveedores(orden=None):
    """Devuelve la lista de proveedores activa según la configuración."""
    if orden is not None:
        return orden

    cascade_env = __import__("os").environ.get("NEXUS_CASCADE_ORDER")
    if cascade_env:
        return [s.strip() for s in cascade_env.split(",") if s.strip()]

    return list(ORDEN_DEFAULT)


# Verificar al importar que ORDEN_DEFAULT solo contiene proveedores registrados
# Esto actúa como una guardia en tiempo de importación
_invalid_providers = [p for p in ORDEN_DEFAULT if p not in MODELOS_PROVEEDOR]
if _invalid_providers:
    raise RuntimeError(
        f"ORDEN_DEFAULT contiene proveedores no registrados: {_invalid_providers}. "
        f"Proveedores disponibles: {list(MODELOS_PROVEEDOR.keys())}"
    )