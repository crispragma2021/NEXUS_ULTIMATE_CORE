"""Integración del ciclo del agente: herramienta execute_cmd end-to-end."""

from __future__ import annotations

import json

import nexus_core_agent as agente
from nexo_api import TransportError


def _mensaje(texto=None, tool_calls=None):
    mensaje = {"role": "assistant"}
    if texto is not None:
        mensaje["content"] = texto
    if tool_calls is not None:
        mensaje["tool_calls"] = tool_calls
    return mensaje


def test_ciclo_con_execute_cmd_real(monkeypatch, tmp_path):
    """El modelo pide execute_cmd, se ejecuta de verdad y se devuelve el resultado."""
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    monkeypatch.setenv("NEXUS_MEM_PROFILE", "normal")

    llamadas = []

    def chat(payload):
        llamadas.append(json.loads(json.dumps(payload)))
        if len(llamadas) == 1:
            return {
                "choices": [
                    {
                        "message": _mensaje(
                            tool_calls=[
                                {
                                    "id": "call_1",
                                    "type": "function",
                                    "function": {
                                        "name": "execute_cmd",
                                        "arguments": json.dumps({"command": "echo NEXUS_OK"}),
                                    },
                                }
                            ]
                        )
                    }
                ],
                "usage": {},
            }
        return {"choices": [{"message": {"content": "listo"}}], "usage": {}}

    resultado = agente.query_agent("haz algo", chat_fn=chat)

    assert resultado == "listo"
    assert len(llamadas) == 2
    mensajes_segunda = llamadas[1]["messages"]
    tool_result = [m for m in mensajes_segunda if m.get("role") == "tool"]
    assert len(tool_result) == 1
    assert tool_result[0]["tool_call_id"] == "call_1"
    assert "NEXUS_OK" in tool_result[0]["content"]


def test_execute_cmd_bloquea_comando_peligroso(monkeypatch, tmp_path):
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    llamadas = []

    def chat(payload):
        llamadas.append(payload)
        if len(llamadas) == 1:
            return {
                "choices": [
                    {
                        "message": _mensaje(
                            tool_calls=[
                                {
                                    "id": "c1",
                                    "function": {
                                        "name": "execute_cmd",
                                        "arguments": json.dumps({"command": "rm -rf /"}),
                                    },
                                }
                            ]
                        )
                    }
                ]
            }
        return {"choices": [{"message": {"content": "bloqueado"}}]}

    agente.query_agent("x", chat_fn=chat)
    tool_msg = [m for m in llamadas[1]["messages"] if m.get("role") == "tool"][0]
    assert tool_msg["content"] == agente.nexo_shell.BLOCKED_MESSAGE


def test_varias_herramientas_usan_un_solo_mensaje_assistant(monkeypatch, tmp_path):
    """No se duplica el contexto por cada tool_call."""
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    llamadas = []

    def chat(payload):
        llamadas.append(payload)
        if len(llamadas) == 1:
            return {
                "choices": [
                    {
                        "message": _mensaje(
                            tool_calls=[
                                {"id": "c1", "function": {"name": "execute_cmd", "arguments": '{"command":"echo 1"}'}},
                                {"id": "c2", "function": {"name": "execute_cmd", "arguments": '{"command":"echo 2"}'}},
                            ]
                        )
                    }
                ]
            }
        return {"choices": [{"message": {"content": "fin"}}]}

    agente.query_agent("x", chat_fn=chat)
    roles = [m.get("role") for m in llamadas[1]["messages"]]
    # Un UNICO mensaje assistant porta los dos tool_calls: no se duplica el
    # contexto (ni se vuelve a pagar tokens) por herramienta invocada.
    assert roles.count("assistant") == 1
    assert roles.count("tool") == 2
    assistant = [m for m in llamadas[1]["messages"] if m.get("role") == "assistant"][0]
    assert len(assistant["tool_calls"]) == 2


def test_arguments_invalidos_no_rompen_el_ciclo(monkeypatch, tmp_path):
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    llamadas = []

    def chat(payload):
        llamadas.append(payload)
        if len(llamadas) == 1:
            return {
                "choices": [
                    {
                        "message": _mensaje(
                            tool_calls=[{"id": "c1", "function": {"name": "execute_cmd", "arguments": "{json roto"}}]
                        )
                    }
                ]
            }
        return {"choices": [{"message": {"content": "ok"}}]}

    assert agente.query_agent("x", chat_fn=chat) == "ok"


def test_herramienta_desconocida_se_reporta(monkeypatch, tmp_path):
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    llamadas = []

    def chat(payload):
        llamadas.append(payload)
        if len(llamadas) == 1:
            return {
                "choices": [
                    {
                        "message": _mensaje(
                            tool_calls=[{"id": "c1", "function": {"name": "formatear_disco", "arguments": "{}"}}]
                        )
                    }
                ]
            }
        return {"choices": [{"message": {"content": "ok"}}]}

    agente.query_agent("x", chat_fn=chat)
    tool_msg = [m for m in llamadas[1]["messages"] if m.get("role") == "tool"][0]
    assert "no soportada" in tool_msg["content"]


def test_el_historial_se_persiste_recortado(monkeypatch, tmp_path):
    ruta = tmp_path / "m.json"
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(ruta))
    monkeypatch.setenv("NEXUS_MEM_PROFILE", "critical")  # max_history_turns = 2

    for i in range(6):
        agente.query_agent(
            "pregunta {}".format(i),
            chat_fn=lambda payload: {"choices": [{"message": {"content": "r"}}]},
        )

    guardado = json.loads(ruta.read_text())
    assert len(guardado) == 4  # 2 turnos x (user + assistant)


def test_respuesta_sin_choices_se_reporta(monkeypatch, tmp_path):
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))
    out = agente.query_agent("x", chat_fn=lambda payload: {"error": "boom"})
    assert out.startswith("Error en API")


def test_error_de_transporte_se_reporta_sin_excepcion(monkeypatch, tmp_path):
    monkeypatch.setenv("NEXUS_MEMORY_FILE", str(tmp_path / "m.json"))

    def falla(payload):
        raise TransportError("cadena agotada")

    out = agente.query_agent("x", chat_fn=falla)
    assert out.startswith("Error de transporte")


def test_system_prompt_estable_para_prompt_caching():
    """El prefijo cacheable no puede cambiar accidentalmente."""
    assert agente.SYSTEM_PROMPT.startswith("Eres el orquestador principal")
    assert len(agente.TOOLS) == 1
    assert agente.TOOLS[0]["function"]["name"] == "execute_cmd"


def test_tools_declaran_command_obligatorio():
    parametros = agente.TOOLS[0]["function"]["parameters"]
    assert parametros["required"] == ["command"]


def test_sin_subprocess_ni_requests_en_el_agente():
    """El agente no debe abrir subprocesos ni HTTP por su cuenta."""
    from pathlib import Path

    texto = (Path(__file__).resolve().parent.parent / "agents" / "nexus_core_agent.py").read_text()
    assert "subprocess.run" not in texto
    assert "requests.post" not in texto
