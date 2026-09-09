"""Tarea 1 — Fallback automático y transparente de modelos en la capa de transporte."""

from __future__ import annotations

import nexo_api
from conftest import (
    fake_gemini_response,
    gemini_error,
    key_of,
    make_transport_script,
    model_of,
)

KEYS = ["key-1", "key-2"]
NO_SLEEP = lambda _seconds: None  # noqa: E731  (acelera los tests)


# --------------------------------------------------------------------------
# Cadena de modelos
# --------------------------------------------------------------------------
def test_cadena_por_defecto_empieza_en_3_8_y_cae_a_3_7():
    chain = nexo_api.model_chain(None, {})
    assert chain[0] == "gemini-3.8-flash"
    assert chain[1] == "gemini-3.7-flash"


def test_cadena_sobreescribible_por_variable_de_entorno():
    env = {"NEXUS_GEMINI_MODEL_CHAIN": "gemini-3.8-flash,gemini-3.7-flash"}
    assert nexo_api.model_chain(None, env) == ["gemini-3.8-flash", "gemini-3.7-flash"]


def test_modelo_explicito_tiene_prioridad_y_no_se_duplica():
    chain = nexo_api.model_chain(
        "gemini-3.8-flash", {"NEXUS_GEMINI_MODEL_CHAIN": "gemini-3.8-flash,gemini-3.7-flash"}
    )
    assert chain == ["gemini-3.8-flash", "gemini-3.7-flash"]


def test_auto_no_aniade_modelo_a_la_cadena():
    assert nexo_api.model_chain("auto", {}) == list(nexo_api.DEFAULT_MODEL_CHAIN)


# --------------------------------------------------------------------------
# Clasificación de errores
# --------------------------------------------------------------------------
def test_503_unavailable_es_reintentable():
    assert nexo_api.is_retryable(503, gemini_error("UNAVAILABLE")) is True


def test_429_resource_exhausted_es_reintentable():
    assert nexo_api.is_retryable(429, gemini_error("RESOURCE_EXHAUSTED")) is True


def test_401_no_es_reintentable():
    """Una llave inválida no debe quemar la cadena entera de modelos."""
    assert nexo_api.is_retryable(401, gemini_error("UNAUTHENTICATED")) is False


def test_estado_200_no_es_reintentable():
    assert nexo_api.is_retryable(200, {}) is False


def test_estado_desconocido_con_status_unavailable_en_el_cuerpo():
    assert nexo_api.is_retryable(500, gemini_error("UNAVAILABLE")) is True


# --------------------------------------------------------------------------
# Comportamiento del transporte
# --------------------------------------------------------------------------
def test_primer_modelo_ok_no_hace_fallback(monkeypatch):
    fake = make_transport_script((200, fake_gemini_response("hola")))
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    result = nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "hi"}]}],
        keys=KEYS,
        env={},
        sleep=NO_SLEEP,
    )

    assert result.model == "gemini-3.8-flash"
    assert result.used_fallback is False
    assert nexo_api.extract_text(result.raw) == "hola"
    assert len(fake.calls) == 1


def test_503_unavailable_hace_fallback_inmediato_a_3_7(monkeypatch):
    """El caso central pedido: 503 UNAVAILABLE -> gemini-3.7-flash."""
    fake = make_transport_script(
        (503, gemini_error("UNAVAILABLE", "Model is overloaded")),
        (200, fake_gemini_response("desde 3.7")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    result = nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "hi"}]}],
        keys=KEYS,
        env={},
        sleep=NO_SLEEP,
    )

    assert result.model == "gemini-3.7-flash"
    assert result.used_fallback is True
    assert [model_of(c["url"]) for c in fake.calls] == [
        "gemini-3.8-flash",
        "gemini-3.7-flash",
    ]
    assert nexo_api.extract_text(result.raw) == "desde 3.7"


def test_429_hace_fallback_a_3_7(monkeypatch):
    fake = make_transport_script(
        (429, gemini_error("RESOURCE_EXHAUSTED")),
        (200, fake_gemini_response("recuperado")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    result = nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "hi"}]}],
        keys=KEYS,
        env={},
        sleep=NO_SLEEP,
        max_attempts_per_model=1,
    )

    assert result.model == "gemini-3.7-flash"
    assert nexo_api.extract_text(result.raw) == "recuperado"


def test_fallback_no_es_transparente_para_el_payload(monkeypatch):
    """El cuerpo enviado es byte a byte el mismo al cambiar de modelo."""
    fake = make_transport_script(
        (503, gemini_error("UNAVAILABLE")),
        (200, fake_gemini_response("ok")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "carga útil con acentos 🚀"}]}],
        keys=KEYS,
        env={},
        sleep=NO_SLEEP,
    )

    assert fake.calls[0]["payload"] == fake.calls[1]["payload"]


def test_401_corta_la_cadena_sin_quemar_todos_los_modelos(monkeypatch):
    fake = make_transport_script((401, gemini_error("UNAUTHENTICATED", "API key not valid")))
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    try:
        nexo_api.call_gemini(
            [{"role": "user", "parts": [{"text": "hi"}]}],
            keys=KEYS,
            env={},
            sleep=NO_SLEEP,
        )
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError as exc:
        assert len(fake.calls) == 1
        assert "UNAUTHENTICATED" in exc.describe()


def test_toda_la_cadena_agotada_lanza_con_trazabilidad(monkeypatch):
    fake = make_transport_script(
        (503, gemini_error("UNAVAILABLE")),
        (503, gemini_error("UNAVAILABLE")),
        (503, gemini_error("UNAVAILABLE")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    try:
        nexo_api.call_gemini(
            [{"role": "user", "parts": [{"text": "hi"}]}],
            keys=["k1"],
            env={"NEXUS_GEMINI_MODEL_CHAIN": "gemini-3.8-flash,gemini-3.7-flash"},
            sleep=NO_SLEEP,
            max_attempts_per_model=1,
        )
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError as exc:
        assert len(exc.attempts) == 2
        assert "gemini-3.8-flash" in exc.describe()
        assert "gemini-3.7-flash" in exc.describe()


def test_error_de_red_no_rompe_el_fallback(monkeypatch):
    fake = make_transport_script(
        TimeoutError("socket timeout"),
        (200, fake_gemini_response("tras timeout")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    result = nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "hi"}]}],
        keys=["k1"],
        env={"NEXUS_GEMINI_MODEL_CHAIN": "gemini-3.8-flash,gemini-3.7-flash"},
        sleep=NO_SLEEP,
    )
    assert result.model == "gemini-3.7-flash"
    assert nexo_api.extract_text(result.raw) == "tras timeout"


def test_llave_con_429_se_penaliza_y_rota(monkeypatch):
    """Tras un 429 la llave queda penalizada y se prueba la siguiente."""
    fake = make_transport_script(
        (429, gemini_error("RESOURCE_EXHAUSTED")),
        (200, fake_gemini_response("otra llave")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    result = nexo_api.call_gemini(
        [{"role": "user", "parts": [{"text": "hi"}]}],
        keys=KEYS,
        env={"NEXUS_GEMINI_MODEL_CHAIN": "gemini-3.8-flash"},
        sleep=NO_SLEEP,
    )

    assert nexo_api.extract_text(result.raw) == "otra llave"
    assert [key_of(c["url"]) for c in fake.calls] == ["key-1", "key-2"]


def test_sin_llaves_falla_con_mensaje_accionable(monkeypatch):
    try:
        nexo_api.call_gemini(
            [{"role": "user", "parts": [{"text": "hi"}]}], keys=[], env={}, sleep=NO_SLEEP
        )
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError as exc:
        assert "NEXUS_GEMINI_KEYS" in str(exc)


def test_load_pool_keys_deduplica_y_lee_varias_variables():
    env = {
        "NEXUS_GEMINI_KEYS": "a b",
        "GEMINI_API_KEY": "b,c",
    }
    assert nexo_api.load_pool_keys(env) == ["a", "b", "c"]


def test_no_hay_llaves_hardcodeadas_en_los_agentes():
    """Regresión: las llaves reales que había en git no pueden volver."""
    import re
    from pathlib import Path

    root = Path(__file__).resolve().parent.parent / "agents"
    patron = re.compile(r"AIza[0-9A-Za-z_\-]{20,}|sk-[0-9a-f]{24,}")
    for archivo in sorted(root.glob("*.py")):
        assert not patron.search(archivo.read_text(encoding="utf-8")), (
            "Llave hardcodeada en {}".format(archivo.name)
        )


def test_backoff_exponencial_acotado():
    delays = [nexo_api.retry_delay(i) for i in range(1, 12)]
    assert all(d >= 0 for d in delays)
    assert max(delays) <= 3000 * 1.3 / 1000.0


# --------------------------------------------------------------------------
# Fachada chat (DeepSeek)
# --------------------------------------------------------------------------
def test_chat_con_fallback_de_modelo(monkeypatch):
    fake = make_transport_script(
        (503, {"error": {"message": "overloaded"}}),
        (200, {"choices": [{"message": {"content": "ok"}}]}),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    res = nexo_api.chat(
        {"model": "deepseek-chat", "messages": []},
        api_key="sk-test",
        models=["deepseek-chat", "deepseek-reasoner"],
        sleep=NO_SLEEP,
    )
    assert res["choices"][0]["message"]["content"] == "ok"


def test_chat_sin_key_falla_rapido():
    try:
        nexo_api.chat({"model": "x"}, api_key="")
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError:
        pass
