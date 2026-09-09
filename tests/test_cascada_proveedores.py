"""Cascada multiproveedor: la redundancia real no es entre llaves, es entre pools.

Regresión central: rotar llaves del mismo proveedor NO suma cuota (Gemini limita
por proyecto y OpenRouter gobierna la capacidad globalmente), así que estos
tests fijan que el tráfico salte de PROVEEDOR y que no se queme cuota a ciegas.
"""

from __future__ import annotations

import json

import pytest

import nexo_api
import nexo_proveedores as prov
from conftest import make_transport_script

NO_SLEEP = lambda _s: None  # noqa: E731

ENV_DOS = {
    "GROQ_API_KEY": "gsk-test",
    "OPENROUTER_API_KEY": "sk-or-test",
    "NEXUS_TRANSPORT_DEBUG": "0",
    "NEXUS_PROVIDER_STATE": "",
}


def respuesta_ok(texto="ok"):
    return {"choices": [{"message": {"role": "assistant", "content": texto}}]}


# --------------------------------------------------------------------------
# Registro y detección
# --------------------------------------------------------------------------
def test_registro_no_tiene_proveedores_duplicados():
    nombres = [p.nombre for p in prov.REGISTRO]
    assert len(nombres) == len(set(nombres))


def test_solo_se_arman_los_que_tienen_llave():
    assert prov.armados({}) == []
    armados = prov.armados({"GROQ_API_KEY": "x"})
    assert [p.nombre for p in armados] == ["groq"]


def test_llave_vacia_no_arma_al_proveedor():
    assert prov.armados({"GROQ_API_KEY": "   "}) == []


def test_orden_de_cascada_respeta_nexus_providers():
    env = dict(ENV_DOS, NEXUS_PROVIDERS="openrouter,groq")
    nombres = [p.nombre for p, _ in prov.cascada(env)]
    assert nombres[0] == "openrouter"
    assert nombres[1].startswith("groq")


def test_proveedores_no_listados_van_al_final():
    env = dict(ENV_DOS, NEXUS_PROVIDERS="openrouter")
    nombres = [p.nombre for p, _ in prov.cascada(env)]
    assert nombres[0] == "openrouter"
    assert "groq" in nombres  # no se pierde, solo se reordena


def test_modelo_sobreescribible_por_proveedor():
    env = dict(ENV_DOS, NEXUS_GROQ_MODEL="mi-modelo")
    modelos = [m for p, m in prov.cascada(env) if p.nombre == "groq"]
    assert modelos == ["mi-modelo"]


# --------------------------------------------------------------------------
# Presupuesto
# --------------------------------------------------------------------------
def test_presupuesto_diario_se_agota():
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    groq = prov.proveedor("groq")
    for _ in range(groq.rpd):
        estado.registrar_exito(groq)
    permitido, motivo = prov.puede_llamar(groq, estado)
    assert permitido is False
    assert "diaria" in motivo


def test_presupuesto_se_reinicia_al_cambiar_el_dia():
    reloj = [1_000_000.0]
    estado = prov.EstadoPresupuesto({}, ahora=lambda: reloj[0])
    groq = prov.proveedor("groq")
    for _ in range(groq.rpd):
        estado.registrar_exito(groq)
    assert prov.puede_llamar(groq, estado)[0] is False

    reloj[0] += 86_400  # un dia despues
    assert prov.puede_llamar(groq, estado)[0] is True


def test_rpm_se_mide_en_ventana_rodante():
    reloj = [1_000_000.0]
    estado = prov.EstadoPresupuesto({}, ahora=lambda: reloj[0])
    groq = prov.proveedor("groq")
    for _ in range(groq.rpm):
        estado.registrar_exito(groq)
    assert "RPM" in prov.puede_llamar(groq, estado)[1]

    reloj[0] += 61  # la ventana rueda
    assert prov.puede_llamar(groq, estado)[0] is True


def test_fallo_pone_al_proveedor_en_cooldown():
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    groq = prov.proveedor("groq")
    estado.registrar_fallo(groq, retry_after=30.0)
    assert estado.en_cooldown(groq) is True
    assert estado.segundos_en_cooldown(groq) == 30


def test_exito_levanta_el_cooldown():
    reloj = [1_000_000.0]
    estado = prov.EstadoPresupuesto({}, ahora=lambda: reloj[0])
    groq = prov.proveedor("groq")
    estado.registrar_fallo(groq, retry_after=30.0)
    estado.registrar_exito(groq)
    assert estado.en_cooldown(groq) is False


def test_un_429_tambien_gasta_cuota():
    """En OpenRouter los intentos fallidos descuentan del presupuesto diario."""
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    groq = prov.proveedor("groq")
    estado.registrar_fallo(groq)
    assert estado.restantes_hoy(groq) == groq.rpd - 1


# --------------------------------------------------------------------------
# Persistencia
# --------------------------------------------------------------------------
def test_estado_se_persiste_y_se_recarga(tmp_path):
    ruta = tmp_path / "estado.json"
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    groq = prov.proveedor("groq")
    estado.registrar_fallo(groq, retry_after=30.0)
    assert prov.guardar_estado(estado, str(ruta)) is True

    recargado = prov.cargar_estado(str(ruta), ahora=lambda: 1_000_000.0)
    assert recargado.en_cooldown(groq) is True


def test_estado_corrupto_no_rompe_nada(tmp_path):
    ruta = tmp_path / "estado.json"
    ruta.write_text("{no es json")
    assert prov.cargar_estado(str(ruta)).to_dict() == {}


def test_estado_no_deja_temporales(tmp_path):
    ruta = tmp_path / "estado.json"
    prov.guardar_estado(prov.EstadoPresupuesto(), str(ruta))
    residuos = [p.name for p in tmp_path.iterdir() if p.name.startswith(".nexus_prov_")]
    assert residuos == []


# --------------------------------------------------------------------------
# Cascada en acción
# --------------------------------------------------------------------------
def test_primer_proveedor_responde_y_no_hay_salto(monkeypatch):
    fake = make_transport_script((200, respuesta_ok("groq responde")))
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    estado = prov.EstadoPresupuesto()

    res, usado, modelo = nexo_api.chat_cascade(
        {"messages": []}, env=ENV_DOS, estado=estado, sleep=NO_SLEEP
    )
    assert usado == "groq"
    assert modelo == "llama-3.3-70b-versatile"
    assert res["choices"][0]["message"]["content"] == "groq responde"
    assert len(fake.calls) == 1


def test_429_salta_al_siguiente_proveedor_no_a_otra_llave(monkeypatch):
    """La regresión central: un 429 enfría al PROVEEDOR entero.

    No se prueba otra llave ni otro modelo del mismo proveedor (la cuota es
    compartida), se salta directamente a un pool distinto.
    """
    fake = make_transport_script(
        (429, {"error": {"message": "Rate limit exceeded"}}),
        (200, respuesta_ok("desde openrouter")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    estado = prov.EstadoPresupuesto()

    # groq tiene 2 modelos; tras el 429 el segundo debe quedar descartado.
    assert len([m for p, m in prov.cascada(ENV_DOS) if p.nombre == "groq"]) == 2

    res, usado, _modelo = nexo_api.chat_cascade(
        {"messages": []}, env=ENV_DOS, estado=estado, sleep=NO_SLEEP
    )
    assert usado == "openrouter"
    assert res["choices"][0]["message"]["content"] == "desde openrouter"
    urls = [c["url"] for c in fake.calls]
    assert len(urls) == 2, "no debe reintentar el segundo modelo de groq"
    assert "api.groq.com" in urls[0] and "openrouter.ai" in urls[1]


def test_proveedor_en_cooldown_se_omite(monkeypatch):
    fake = make_transport_script((200, respuesta_ok()))
    monkeypatch.setattr(nexo_api, "_http_post", fake)

    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    estado.registrar_fallo(prov.proveedor("groq"), retry_after=60.0)

    _res, usado, _m = nexo_api.chat_cascade(
        {"messages": []}, env=ENV_DOS, estado=estado, sleep=NO_SLEEP
    )
    assert usado == "openrouter"  # groq estaba enfriado


def test_retry_after_del_proveedor_se_respeta():
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    groq = prov.proveedor("groq")
    estado.registrar_fallo(groq, retry_after=120.0)
    assert estado.segundos_en_cooldown(groq) == 120


def test_retry_after_se_extrae_de_la_cabecera():
    class Cabeceras(dict):
        pass

    assert nexo_api.retry_after_seconds(429, {}, Cabeceras({"Retry-After": "45"})) == 45.0


def test_retry_after_se_extrae_del_cuerpo():
    assert nexo_api.retry_after_seconds(429, {"error": {"retry_after": 12}}) == 12.0


def test_sin_ningun_proveedor_armado_falla_con_detalle(monkeypatch):
    fake = make_transport_script()
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    try:
        nexo_api.chat_cascade({"messages": []}, env={"NEXUS_TRANSPORT_DEBUG": "0"}, sleep=NO_SLEEP)
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError as exc:
        assert "Ningun proveedor" in str(exc)


def test_todos_los_proveedores_agotados_falla(monkeypatch):
    fake = make_transport_script(
        (429, {"error": {"message": "limit"}}),
        (429, {"error": {"message": "limit"}}),
        (429, {"error": {"message": "limit"}}),
        (429, {"error": {"message": "limit"}}),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    try:
        nexo_api.chat_cascade(
            {"messages": []}, env=ENV_DOS, estado=prov.EstadoPresupuesto(), sleep=NO_SLEEP
        )
        raise AssertionError("Debería haber lanzado TransportError")
    except nexo_api.TransportError as exc:
        assert len(exc.attempts) >= 2


def test_error_de_red_pasa_al_siguiente_proveedor(monkeypatch):
    fake = make_transport_script(
        ConnectionError("dns caido"),
        (200, respuesta_ok("recuperado")),
    )
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    _res, usado, _m = nexo_api.chat_cascade(
        {"messages": []}, env=ENV_DOS, estado=prov.EstadoPresupuesto(), sleep=NO_SLEEP
    )
    assert usado in ("groq", "openrouter")


def test_la_llave_viaja_en_la_cabecera_no_en_la_url(monkeypatch):
    fake = make_transport_script((200, respuesta_ok()))
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    nexo_api.chat_cascade(
        {"messages": []}, env=ENV_DOS, estado=prov.EstadoPresupuesto(), sleep=NO_SLEEP
    )
    llamada = fake.calls[0]
    assert llamada["headers"]["Authorization"].startswith("Bearer ")
    assert "gsk-test" not in llamada["url"], "la llave no debe ir en la URL"


def test_el_modelo_se_inyecta_en_el_cuerpo(monkeypatch):
    fake = make_transport_script((200, respuesta_ok()))
    monkeypatch.setattr(nexo_api, "_http_post", fake)
    nexo_api.chat_cascade(
        {"messages": [{"role": "user", "content": "h"}], "model": "ignorado"},
        env=ENV_DOS,
        estado=prov.EstadoPresupuesto(),
        sleep=NO_SLEEP,
    )
    cuerpo = json.loads(fake.calls[0]["payload"])
    assert cuerpo["model"] == "llama-3.3-70b-versatile"
    assert cuerpo["messages"] == [{"role": "user", "content": "h"}]


# --------------------------------------------------------------------------
# Diagnóstico
# --------------------------------------------------------------------------
def test_resumen_lista_todo_el_registro():
    datos = prov.resumen({"GROQ_API_KEY": "x"}, prov.EstadoPresupuesto())
    assert len(datos) == len(prov.REGISTRO)
    groq = next(d for d in datos if d["nombre"] == "groq")
    assert groq["armado"] is True and groq["listo"] is True
    gemini = next(d for d in datos if d["nombre"] == "gemini")
    assert gemini["armado"] is False


def test_cadena_cascada_omite_proveedores_enfriados():
    estado = prov.EstadoPresupuesto({}, ahora=lambda: 1_000_000.0)
    estado.registrar_fallo(prov.proveedor("groq"), retry_after=60.0)
    cadena = prov.cadena_cascada(ENV_DOS, estado)
    assert "groq" not in cadena
    assert "openrouter" in cadena


def test_cadena_cascada_sin_proveedores():
    assert prov.cadena_cascada({}, prov.EstadoPresupuesto()).startswith("(ningun")
