"""
nexo_api — Capa de transporte resiliente hacia Gemini.

Responsabilidades:
  1. Cadena de fallback de modelos. Si ``gemini-3.8-flash`` responde 503
     UNAVAILABLE / 429 RESOURCE_EXHAUSTED / 500 INTERNAL (overload), el reintento
     salta INMEDIATO y TRANSPARENTE al siguiente modelo de la cadena
     (``gemini-3.7-flash``) sin coste de backoff.
  2. Rotación de llaves del pool con penalización temporal de las llaves que
     agotan cuota (429), evitando re-golpear una llave muerta.
  3. Backoff exponencial con jitter SOLO para errores de red.

Diseño para Android/aarch64: sin dependencias obligatorias (usa ``requests`` si
está y ``urllib`` si no), sin hilos, sin reintentos en paralelo.
"""

from __future__ import annotations

import json
import os
import random
import time
import urllib.error
import urllib.request
from typing import Any, Dict, List, Optional, Sequence

__all__ = [
    "GEMINI_ENDPOINT",
    "DEFAULT_MODEL_CHAIN",
    "DEFAULT_MODEL",
    "RETRYABLE_STATUS",
    "RETRYABLE_REASON_CODES",
    "TransportError",
    "Attempt",
    "TransportResult",
    "load_pool_keys",
    "model_chain",
    "retry_delay",
    "is_retryable",
    "model_supports_images",
    "extract_text",
    "call_gemini",
    "chat",
]

GEMINI_ENDPOINT = "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"

# Cadena de fallback por defecto. Sobreescribible con:
#   NEXUS_GEMINI_MODEL_CHAIN="gemini-3.8-flash,gemini-3.7-flash,gemini-1.5-flash"
DEFAULT_MODEL_CHAIN: tuple = (
    "gemini-3.8-flash",
    "gemini-3.7-flash",
    "gemini-1.5-flash",
)
DEFAULT_MODEL = "gemini-3.8-flash"

#: Códigos HTTP que provocan salto inmediato al siguiente modelo/llave.
RETRYABLE_STATUS: frozenset = frozenset({429, 500, 502, 503, 504})

#: Fallos del MODELO (está caído o saturado): saltar de modelo al instante, sin
#: gastar más llaves. Es el caso pedido: 503 UNAVAILABLE -> gemini-3.7-flash.
MODEL_LEVEL_STATUS: frozenset = frozenset({500, 502, 503, 504})
MODEL_LEVEL_REASONS: frozenset = frozenset({"UNAVAILABLE", "OVERLOADED", "INTERNAL"})

#: Fallos de la LLAVE (cuota agotada): rotar de llave antes que de modelo.
KEY_LEVEL_STATUS: frozenset = frozenset({429})
KEY_LEVEL_REASONS: frozenset = frozenset({"RESOURCE_EXHAUSTED"})

#: Errores de configuración: cambiar de modelo no los arregla, abortar la cadena.
FATAL_STATUS: frozenset = frozenset({400, 401, 403})
FATAL_REASONS: frozenset = frozenset(
    {"UNAUTHENTICATED", "PERMISSION_DENIED", "INVALID_ARGUMENT", "FAILED_PRECONDITION"}
)

#: Motivos declarados por la API que también se tratan como agotamiento temporal.
RETRYABLE_REASON_CODES: frozenset = frozenset(
    {"UNAVAILABLE", "RESOURCE_EXHAUSTED", "INTERNAL", "OVERLOADED", "DEADLINE_EXCEEDED"}
)

#: Modelos de la cadena con entrada multimodal (inline_data) confirmada.
VISION_CAPABLE_PREFIXES: tuple = ("gemini-1.5", "gemini-2.", "gemini-3.")


# --------------------------------------------------------------------------
# Errores
# --------------------------------------------------------------------------
class TransportError(RuntimeError):
    """Error final de la capa de transporte, con trazabilidad de intentos."""

    def __init__(self, message: str, attempts: Optional[List["Attempt"]] = None):
        super().__init__(message)
        self.attempts: List["Attempt"] = list(attempts or [])

    def describe(self) -> str:
        if not self.attempts:
            return str(self)
        return "{} [{}]".format(self, " -> ".join(a.describe() for a in self.attempts))


class Attempt:
    __slots__ = ("model", "status", "reason", "key_index")

    def __init__(self, model: str, status: int, reason: str, key_index: int = 0):
        self.model = model
        self.status = status
        self.reason = reason
        self.key_index = key_index

    def describe(self) -> str:
        return "{}:{}".format(self.model, self.reason or self.status)


class TransportResult:
    """Resultado de un ``call_gemini``/``chat`` exitoso."""

    __slots__ = ("model", "raw", "attempts")

    def __init__(self, model: str, raw: Any, attempts: Sequence[Attempt]):
        self.model = model
        self.raw = raw
        self.attempts = list(attempts)

    @property
    def used_fallback(self) -> bool:
        """True si el modelo que respondió NO fue el primero de la cadena."""
        return len(self.attempts) > 1


# --------------------------------------------------------------------------
# Configuración
# --------------------------------------------------------------------------
def _split_env(raw: str) -> List[str]:
    return [p.strip() for p in raw.replace(";", ",").split(",") if p.strip()]


def default_chain(env: Optional[dict] = None) -> tuple:
    env = os.environ if env is None else env
    raw = str(env.get("NEXUS_GEMINI_MODEL_CHAIN", "")).strip()
    if raw:
        chain = tuple(_split_env(raw))
        if chain:
            return chain
    return DEFAULT_MODEL_CHAIN


def load_pool_keys(
    env: Optional[dict] = None, extra_names: Sequence[str] = ()
) -> List[str]:
    """Llaves del Zenith Pool desde el entorno (nunca hardcodeadas).

    Acepta ``NEXUS_GEMINI_KEYS`` o ``GEMINI_API_KEY`` (una o varias separadas por
    coma/espacio) más los nombres extra que pida cada agente.
    """
    env = os.environ if env is None else env
    names = ["NEXUS_GEMINI_KEYS", "GEMINI_API_KEY", *extra_names]
    out: List[str] = []
    for name in names:
        raw = str(env.get(name, "")).strip()
        if raw:
            normalizado = raw.replace(";", " ").replace(",", " ")
            out.extend(normalizado.split())
    seen, unique = set(), []
    for key in out:
        if key and key not in seen:
            seen.add(key)
            unique.append(key)
    return unique


def model_chain(requested: Optional[str] = None, env: Optional[dict] = None) -> List[str]:
    """Cadena ordenada y sin duplicados que se intentará, en este orden."""
    env = os.environ if env is None else env
    base = list(default_chain(env))
    requested = (requested or "").strip()
    if requested and requested.lower() != "auto":
        base = [requested, *base]
    out: List[str] = []
    for model in base:
        model = model.strip()
        if model and model not in out:
            out.append(model)
    return out


def retry_delay(attempt: int, base_ms: float = 150.0, max_ms: float = 3000.0) -> float:
    """Backoff exponencial con jitter, en segundos."""
    exponent = max(0, int(attempt) - 1)
    raw = base_ms * (2.0 ** min(exponent, 5))
    jittered = min(raw, max_ms) * random.uniform(0.7, 1.3)
    return max(0.0, jittered / 1000.0)


# --------------------------------------------------------------------------
# Clasificación de errores
# --------------------------------------------------------------------------
def _error_reason(body: Any) -> str:
    if isinstance(body, dict):
        err = body.get("error")
        if isinstance(err, dict):
            status = str(err.get("status") or "").upper()
            if status:
                return status
            message = str(err.get("message") or "")
            if message:
                return message.split("\n", 1)[0][:120]
    return "UNKNOWN"


def is_retryable(status: Optional[int], body: Any = None) -> bool:
    """True si el fallo es agotamiento temporal y conviene cambiar modelo/llave."""
    if status in RETRYABLE_STATUS:
        return True
    reason = _error_reason(body).upper()
    return reason in RETRYABLE_REASON_CODES


def classify_failure(status: Optional[int], body: Any = None) -> str:
    """Clasifica un fallo para decidir QUÉ hay que cambiar.

    ``'fatal'``  -> llave/configuración inválida: abortar toda la cadena.
    ``'modelo'`` -> el modelo está caído o saturado: saltar al siguiente modelo
                    de inmediato, sin gastar más llaves.
    ``'llave'``  -> cuota agotada: rotar de llave y seguir en el mismo modelo.
    ``'red'``    -> fallo de transporte local: reintentar con backoff.
    """
    reason = _error_reason(body).upper()
    if status in FATAL_STATUS or reason in FATAL_REASONS:
        return "fatal"
    if status in MODEL_LEVEL_STATUS or reason in MODEL_LEVEL_REASONS:
        return "modelo"
    if status in KEY_LEVEL_STATUS or reason in KEY_LEVEL_REASONS:
        return "llave"
    if status is None or status == 0:
        return "red"
    return "modelo"


def model_supports_images(model: str) -> bool:
    model = (model or "").lower()
    return model.startswith(VISION_CAPABLE_PREFIXES)


# --------------------------------------------------------------------------
# HTTP (requests si existe, urllib si no)
# --------------------------------------------------------------------------
def _http_post(url: str, payload: bytes, headers: Dict[str, str], timeout: float):
    """POST JSON. Devuelve ``(status, cuerpo_decodificado)``.

    Punto único de monkeypatch en los tests.
    """
    try:
        import requests  # type: ignore
    except ImportError:
        request = urllib.request.Request(
            url, data=payload, headers=headers, method="POST"
        )
        try:
            with urllib.request.urlopen(request, timeout=timeout) as resp:
                return resp.status, json.loads(resp.read().decode("utf-8"))
        except urllib.error.HTTPError as exc:
            raw = exc.read().decode("utf-8", errors="replace")
            try:
                return exc.code, json.loads(raw)
            except ValueError:
                return exc.code, {"error": {"message": raw[:400]}}

    resp = requests.post(url, data=payload, headers=headers, timeout=timeout)
    text = resp.text
    try:
        return resp.status_code, json.loads(text)
    except ValueError:
        return resp.status_code, {"error": {"message": text[:400]}}


# --------------------------------------------------------------------------
# Extracción de respuesta
# --------------------------------------------------------------------------
def extract_text(raw: Any) -> str:
    """Concatena el texto de todas las partes del primer candidato."""
    if not isinstance(raw, dict):
        return ""
    candidates = raw.get("candidates")
    if not candidates:
        return ""
    parts = (candidates[0].get("content") or {}).get("parts") or []
    chunks = [p.get("text") for p in parts if isinstance(p, dict)]
    return "".join(c for c in chunks if c)


# --------------------------------------------------------------------------
# Núcleo del transporte
# --------------------------------------------------------------------------
def call_gemini(
    contents: Sequence[Dict[str, Any]],
    *,
    model: Optional[str] = None,
    keys: Optional[Sequence[str]] = None,
    env: Optional[dict] = None,
    generation_config: Optional[Dict[str, Any]] = None,
    timeout: Optional[float] = None,
    sleep=time.sleep,
    max_attempts_per_model: Optional[int] = None,
    payload: Optional[bytes] = None,
) -> TransportResult:
    """Ejecuta ``generateContent`` recorriendo la cadena de modelos y el pool de llaves.

    El payload JSON se serializa UNA sola vez y se reutiliza en todos los
    intentos: el ``inline_data`` base64 de una imagen nunca se vuelve a
    codificar ni a copiar al cambiar de modelo. Si el llamante ya tiene el
    cuerpo listo (``payload``, ver ``nexo_vision.build_payload_bytes``), se usa
    tal cual y no se serializa nada.
    """
    env = os.environ if env is None else env
    keys = list(keys) if keys is not None else load_pool_keys(env)
    if not keys:
        raise TransportError(
            "Sin llaves API: exporta NEXUS_GEMINI_KEYS o GEMINI_API_KEY "
            "(ver .env.example). No se guardan llaves en el código."
        )

    timeout = float(timeout or env.get("NEXUS_GEMINI_TIMEOUT", 30))
    debug = str(env.get("NEXUS_TRANSPORT_DEBUG", "1")) not in ("0", "false", "False")
    chain = model_chain(model, env)

    attempts: List[Attempt] = []
    penalised: Dict[int, float] = {}
    now = getattr(time, "monotonic", time.time)

    for index, current_model in enumerate(chain):
        attempts_for_model = max_attempts_per_model or min(len(keys), 2)
        key_cursor = 0
        tried = 0

        while tried < attempts_for_model:
            # Saltar llaves penalizadas por 429 reciente.
            chosen = None
            for offset in range(len(keys)):
                candidate = (key_cursor + offset) % len(keys)
                if now() >= penalised.get(candidate, 0.0):
                    chosen = candidate
                    break
            if chosen is None:
                break
            key_cursor = (chosen + 1) % len(keys)
            tried += 1

            url = "{}?key={}".format(GEMINI_ENDPOINT.format(model=current_model), keys[chosen])
            if payload is None:
                body = {
                    "contents": list(contents),
                    "generationConfig": dict(generation_config or {"temperature": 0.4}),
                }
                payload = json.dumps(body).encode("utf-8")
            headers = {"Content-Type": "application/json"}

            try:
                status, parsed = _http_post(url, payload, headers, timeout)
            except Exception as exc:  # red caída, DNS, timeout de socket...
                reason = type(exc).__name__
                attempts.append(Attempt(current_model, 0, reason, chosen))
                if debug:
                    print("[transport] red {} en {}".format(reason, current_model))
                sleep(retry_delay(len(attempts)))
                continue

            if status == 200 and isinstance(parsed, dict) and parsed.get("candidates"):
                attempts.append(Attempt(current_model, 200, "OK", chosen))
                return TransportResult(current_model, parsed, attempts)

            reason = _error_reason(parsed) if isinstance(parsed, dict) else "EMPTY"
            attempts.append(Attempt(current_model, status, reason, chosen))
            clase = classify_failure(status, parsed)
            if debug:
                print(
                    "[transport] {} -> {} ({}, {}) llave#{}".format(
                        current_model, status, reason, clase, chosen
                    )
                )

            if clase == "fatal":
                raise TransportError(
                    "Error de configuracion en {}: {} ({})".format(
                        current_model, reason, status
                    ),
                    attempts,
                )
            if clase == "llave":
                # Cuota agotada: penalizar la llave 30 s y probar la siguiente.
                penalised[chosen] = now() + 30.0
                continue
            # clase == "modelo": el modelo no responde, salto inmediato al siguiente.
            break

        # Agotado este modelo: el salto al siguiente es INMEDIATO (sin backoff).
        if index + 1 < len(chain) and debug:
            print("[transport] fallback {} -> {}".format(current_model, chain[index + 1]))

    raise TransportError("Cadena de modelos agotada", attempts)


# --------------------------------------------------------------------------
# Fachada estilo OpenAI (DeepSeek u otro proveedor compatible)
# --------------------------------------------------------------------------
def chat(
    payload: Dict[str, Any],
    *,
    url: Optional[str] = None,
    api_key: str,
    timeout: float = 30.0,
    models: Optional[Sequence[str]] = None,
    sleep=time.sleep,
) -> Any:
    """``chat/completions`` con la misma política de fallback que ``call_gemini``.

    ``models`` permite degradar de modelo dentro del mismo proveedor (por
    ejemplo ``deepseek-chat`` -> ``deepseek-reasoner``) ante 429/503.
    """
    if not api_key:
        raise TransportError("Sin API key para el proveedor chat.")
    base_url = url or "https://api.deepseek.com/v1/chat/completions"
    headers = {
        "Authorization": "Bearer {}".format(api_key),
        "Content-Type": "application/json",
    }

    attempts: List[Attempt] = []
    body = dict(payload)
    chain = [str(m) for m in (models or [])] or [str(body.get("model", "deepseek-chat"))]

    for current_model in chain:
        body["model"] = current_model
        raw = json.dumps(body).encode("utf-8")
        for tried in range(2):
            try:
                status, parsed = _http_post(base_url, raw, headers, timeout)
            except Exception as exc:
                attempts.append(Attempt(current_model, 0, type(exc).__name__))
                sleep(retry_delay(len(attempts)))
                continue
            if status == 200 and isinstance(parsed, dict) and parsed.get("choices"):
                attempts.append(Attempt(current_model, 200, "OK"))
                return parsed
            reason = _error_reason(parsed) if isinstance(parsed, dict) else "EMPTY"
            attempts.append(Attempt(current_model, status, reason))
            if not is_retryable(status, parsed):
                break
    raise TransportError("Proveedor chat agotado", attempts)
