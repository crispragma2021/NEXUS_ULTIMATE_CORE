#!/usr/bin/env python3
"""
nexo_proveedores — Cascada multiproveedor para que el agente no se quede sin
combustible.

POR QUÉ EXISTE
--------------
Rotar llaves dentro de un mismo proveedor NO da más cuota:

  * Gemini Developer API aplica los límites POR PROYECTO, no por llave. Trece
    llaves del mismo proyecto de AI Studio comparten el mismo pool; crear más
    llaves no suma nada y el patrón de rotación rápida desde una sola IP se ve
    como abuso, que es justo lo que provoca los bloqueos.
  * OpenRouter lo dice literalmente en su documentación: "Making additional
    accounts or API keys will not affect your rate limits, as we govern capacity
    globally".

La redundancia real se consigue encadenando PROVEEDORES distintos, cada uno con
su propio pool de cuota. Este módulo mantiene el registro de proveedores, su
presupuesto y sus enfriamientos, y decide a quién llamar.

El estado se persiste en disco: el agente es un proceso corto que nace por cada
consulta, y sin estado compartido cada arranque volvería a quemar cuota contra
un proveedor que ya sabemos que está agotado.

TRAMPA IMPORTANTE
-----------------
Activar la facturación en un proyecto de Gemini que usa el nivel gratuito hace
desaparecer el nivel gratuito. Si dependes de la cuota gratis, NO vincules una
tarjeta a ese proyecto.
"""

from __future__ import annotations

import json
import os
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Sequence, Tuple

__all__ = [
    "Proveedor",
    "REGISTRO",
    "proveedor",
    "armados",
    "cascada",
    "EstadoPresupuesto",
    "cargar_estado",
    "guardar_estado",
    "puede_llamar",
    "resumen",
    "cadena_cascada",
    "orden_preferido",
    "estado_path",
]


@dataclass(frozen=True)
class Proveedor:
    """Un proveedor de inferencia y su presupuesto gratuito conocido."""

    nombre: str
    base_url: str
    var_key: str
    modelos: Tuple[str, ...]
    rpm: int
    rpd: int
    herramientas: bool = True
    vision: bool = False
    contexto: int = 128_000
    notas: str = ""

    def llave(self, env: Optional[dict] = None) -> str:
        env = os.environ if env is None else env
        return str(env.get(self.var_key, "")).strip()

    def disponible(self, env: Optional[dict] = None) -> bool:
        return bool(self.llave(env))


# --------------------------------------------------------------------------
# Registro de proveedores gratuitos (verificado en septiembre de 2026)
#
# Los límites cambian: trátalos como presupuesto orientativo para no quemar
# cuota a ciegas, no como un contrato. La fuente de verdad es la consola de cada
# proveedor.
# --------------------------------------------------------------------------
REGISTRO: Tuple[Proveedor, ...] = (
    Proveedor(
        nombre="deepseek",
        base_url="https://api.deepseek.com/v1/chat/completions",
        var_key="DEEPSEEK_API_KEY",
        modelos=("deepseek-chat", "deepseek-reasoner"),
        rpm=600,
        rpd=100_000,
        herramientas=True,
        vision=False,
        contexto=64_000,
        notas="De pago pero muy barato (<$0.30/M tokens). Es el que ya usas.",
    ),
    Proveedor(
        nombre="groq",
        base_url="https://api.groq.com/openai/v1/chat/completions",
        var_key="GROQ_API_KEY",
        modelos=("llama-3.3-70b-versatile", "llama-3.1-8b-instant"),
        rpm=30,
        rpd=1_000,
        herramientas=True,
        vision=False,
        contexto=128_000,
        notas="30 RPM / 1.000 RPD por modelo / 6K TPM. Sin tarjeta. Muy rapido (LPU).",
    ),
    Proveedor(
        nombre="cerebras",
        base_url="https://api.cerebras.ai/v1/chat/completions",
        var_key="CEREBRAS_API_KEY",
        modelos=("llama3.3-70b", "qwen-3-32b"),
        rpm=30,
        rpd=14_400,
        herramientas=True,
        vision=False,
        contexto=8_192,  # el nivel gratuito tiene el contexto recortado
        notas="1M de tokens/dia, pero contexto de 8K en el nivel gratuito.",
    ),
    Proveedor(
        nombre="github",
        base_url="https://models.inference.ai.azure.com/chat/completions",
        var_key="GITHUB_TOKEN",
        modelos=("gpt-4.1-mini", "Llama-3.1-8B-Instruct"),
        rpm=15,
        rpd=150,
        herramientas=True,
        vision=False,
        contexto=128_000,
        notas="15 RPM / 150 RPD con Copilot Free. Ya tienes cuenta de GitHub.",
    ),
    Proveedor(
        nombre="openrouter",
        base_url="https://openrouter.ai/api/v1/chat/completions",
        var_key="OPENROUTER_API_KEY",
        modelos=("openrouter/auto",),
        rpm=20,
        rpd=50,
        herramientas=True,
        vision=True,
        contexto=131_072,
        notas="20 RPM / 50 RPD; sube a 1.000 RPD con una compra unica de $10.",
    ),
    Proveedor(
        nombre="gemini",
        base_url="https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
        var_key="GEMINI_API_KEY",
        modelos=("gemini-2.5-flash", "gemini-2.5-flash-lite"),
        rpm=15,
        rpd=1_500,
        herramientas=True,
        vision=True,
        contexto=1_000_000,
        notas="Una sola llave por proyecto: rotar llaves NO suma cuota. Vision nativa.",
    ),
    Proveedor(
        nombre="ollama",
        base_url="http://127.0.0.1:11434/v1/chat/completions",
        var_key="NEXUS_OLLAMA_URL",
        modelos=("qwen2.5:3b",),
        rpm=10_000,
        rpd=10_000_000,
        herramientas=True,
        vision=False,
        contexto=32_000,
        notas="Red local: no depende de cuotas. Ultimo recurso para no quedarse seco.",
    ),
)


def proveedor(nombre: str) -> Optional[Proveedor]:
    for candidato in REGISTRO:
        if candidato.nombre == nombre:
            return candidato
    return None


def armados(env: Optional[dict] = None) -> List[Proveedor]:
    """Proveedores con credencial configurada, en el orden del registro."""
    env = os.environ if env is None else env
    return [p for p in REGISTRO if p.disponible(env)]


def orden_preferido(env: Optional[dict] = None) -> List[Proveedor]:
    """Orden de cascada: lo que pida NEXUS_PROVIDERS y luego el resto armado."""
    env = os.environ if env is None else env
    disponibles = armados(env)
    crudo = str(env.get("NEXUS_PROVIDERS", "")).strip()
    if not crudo:
        return disponibles
    pedidos = [n.strip().lower() for n in crudo.replace(";", ",").split(",") if n.strip()]
    ordenados = [p for n in pedidos for p in disponibles if p.nombre == n]
    ordenados += [p for p in disponibles if p not in ordenados]
    return ordenados


def cascada(env: Optional[dict] = None) -> List[Tuple[Proveedor, str]]:
    """Pares (proveedor, modelo) en el orden en que se intentarán."""
    env = os.environ if env is None else env
    salidos: List[Tuple[Proveedor, str]] = []
    for prov in orden_preferido(env):
        override = str(env.get("NEXUS_{}_MODEL".format(prov.nombre.upper()), "")).strip()
        modelos = (override,) if override else prov.modelos
        for modelo in modelos:
            salidos.append((prov, modelo))
    return salidos


# --------------------------------------------------------------------------
# Presupuesto persistido
# --------------------------------------------------------------------------
class EstadoPresupuesto:
    """Contadores por proveedor con ventanas de minuto y día (UTC)."""

    def __init__(self, datos: Optional[Dict[str, Any]] = None, ahora: Optional[float] = None):
        self.datos: Dict[str, Any] = dict(datos or {})
        self.ahora = ahora if ahora is not None else time.time

    # ------------------------------------------------------------------
    def _entrada(self, nombre: str) -> Dict[str, Any]:
        return self.datos.setdefault(
            nombre,
            {
                "dia": "",
                "usadas_dia": 0,
                "ventana": [],
                "cooldown_hasta": 0.0,
                "fallos": 0,
            },
        )

    def _dia_utc(self) -> str:
        return time.strftime("%Y-%m-%d", time.gmtime(self.ahora()))

    # ------------------------------------------------------------------
    def restantes_hoy(self, prov: Proveedor) -> int:
        entrada = self._entrada(prov.nombre)
        if entrada.get("dia") != self._dia_utc():
            return prov.rpd
        return max(0, prov.rpd - int(entrada.get("usadas_dia", 0)))

    def en_cooldown(self, prov: Proveedor) -> bool:
        return self.ahora() < float(self._entrada(prov.nombre).get("cooldown_hasta", 0.0))

    def segundos_en_cooldown(self, prov: Proveedor) -> int:
        falta = float(self._entrada(prov.nombre).get("cooldown_hasta", 0.0)) - self.ahora()
        return max(0, int(falta))

    # ------------------------------------------------------------------
    def registrar_exito(self, prov: Proveedor) -> None:
        entrada = self._entrada(prov.nombre)
        if entrada.get("dia") != self._dia_utc():
            entrada["dia"] = self._dia_utc()
            entrada["usadas_dia"] = 0
        entrada["usadas_dia"] = int(entrada.get("usadas_dia", 0)) + 1
        entrada["ventana"] = self._ventana_reciente(entrada) + [self.ahora()]
        entrada["fallos"] = 0
        entrada["cooldown_hasta"] = 0.0

    def registrar_fallo(
        self, prov: Proveedor, *, agotado: bool = False, retry_after: Optional[float] = None
    ) -> None:
        """Un 429 también gasta cuota: se cuenta y se enfría al proveedor."""
        entrada = self._entrada(prov.nombre)
        if entrada.get("dia") != self._dia_utc():
            entrada["dia"] = self._dia_utc()
            entrada["usadas_dia"] = 0
        entrada["usadas_dia"] = int(entrada.get("usadas_dia", 0)) + 1
        entrada["fallos"] = int(entrada.get("fallos", 0)) + 1

        if retry_after is not None and retry_after > 0:
            espera = float(retry_after)
        elif agotado:
            # Cuota diaria agotada: enfriar hasta el próximo medianoche UTC.
            espera = self._segundos_hasta_medianoche()
        else:
            espera = min(120.0, 5.0 * (2 ** min(entrada["fallos"], 4)))
        entrada["cooldown_hasta"] = self.ahora() + espera

    def _segundos_hasta_medianoche(self) -> float:
        ahora = self.ahora()
        return max(1.0, 86_400.0 - (ahora % 86_400.0))

    def _ventana_reciente(self, entrada: Dict[str, Any]) -> List[float]:
        corte = self.ahora() - 60.0
        return [t for t in entrada.get("ventana", []) if t > corte]

    def rpm_usados(self, prov: Proveedor) -> int:
        entrada = self._entrada(prov.nombre)
        entrada["ventana"] = self._ventana_reciente(entrada)
        return len(entrada["ventana"])

    # ------------------------------------------------------------------
    def to_dict(self) -> Dict[str, Any]:
        return self.datos


def estado_path(env: Optional[dict] = None) -> str:
    env = os.environ if env is None else env
    override = env.get("NEXUS_PROVIDER_STATE")
    if override:
        return override
    import nexo_plataforma as plataforma

    return plataforma.temp_path("nexus_proveedores_estado.json", env=env)


def cargar_estado(
    path: Optional[str] = None,
    env: Optional[dict] = None,
    ahora=None,
) -> EstadoPresupuesto:
    """Reloj inyectable: sin él los tests de cooldown persistido no son fiables."""
    ruta = path or estado_path(env)
    try:
        with open(ruta, "r", encoding="utf-8") as fh:
            datos = json.load(fh)
        if not isinstance(datos, dict):
            datos = {}
    except (OSError, ValueError):
        datos = {}
    return EstadoPresupuesto(datos, ahora=ahora)


def guardar_estado(estado: EstadoPresupuesto, path: Optional[str] = None, env: Optional[dict] = None) -> bool:
    ruta = path or estado_path(env)
    import tempfile

    try:
        directorio = os.path.dirname(ruta) or "."
        os.makedirs(directorio, exist_ok=True)
        blob = json.dumps(estado.to_dict(), separators=(",", ":")).encode("utf-8")
        handle, temporal = tempfile.mkstemp(dir=directorio, prefix=".nexus_prov_", suffix=".tmp")
        try:
            with os.fdopen(handle, "wb") as fh:
                fh.write(blob)
            os.replace(temporal, ruta)
        except BaseException:
            if os.path.exists(temporal):
                os.unlink(temporal)
            raise
    except OSError:
        return False
    return True


# --------------------------------------------------------------------------
# Decisión
# --------------------------------------------------------------------------
def puede_llamar(
    prov: Proveedor, estado: EstadoPresupuesto
) -> Tuple[bool, str]:
    """Devuelve (permitido, motivo). Evita gastar cuota en llamadas condenadas."""
    if estado.en_cooldown(prov):
        return False, "enfriado {}s".format(estado.segundos_en_cooldown(prov))
    if estado.restantes_hoy(prov) <= 0:
        return False, "cuota diaria agotada"
    if estado.rpm_usados(prov) >= prov.rpm:
        return False, "limite de {} RPM alcanzado".format(prov.rpm)
    return True, "ok"


def resumen(env: Optional[dict] = None, estado: Optional[EstadoPresupuesto] = None) -> List[Dict[str, Any]]:
    """Estado de cada proveedor, para `nexus doctor`."""
    env = os.environ if env is None else env
    estado = estado or cargar_estado(env=env)
    salida = []
    for prov in REGISTRO:
        permitido, motivo = puede_llamar(prov, estado)
        salida.append(
            {
                "nombre": prov.nombre,
                "armado": prov.disponible(env),
                "listo": prov.disponible(env) and permitido,
                "motivo": motivo,
                "restantes_hoy": estado.restantes_hoy(prov),
                "rpm_usados": estado.rpm_usados(prov),
                "modelos": list(prov.modelos),
                "notas": prov.notas,
            }
        )
    return salida


def cadena_cascada(env: Optional[dict] = None, estado: Optional[EstadoPresupuesto] = None) -> str:
    """Descripción legible de por dónde irá el tráfico ahora mismo."""
    env = os.environ if env is None else env
    estado = estado or cargar_estado(env=env)
    partes = []
    for prov, modelo in cascada(env):
        permitido, motivo = puede_llamar(prov, estado)
        if not permitido:
            continue
        partes.append("{}:{}".format(prov.nombre, modelo))
    return " -> ".join(partes) if partes else "(ningun proveedor disponible)"
