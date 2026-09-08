"""Tarea 3 — Pipeline multimodal inline_data sin lecturas intermedias redundantes."""

from __future__ import annotations

import base64
import io
import json
import os
from pathlib import Path

import pytest

import nexo_vision as vision


@pytest.fixture()
def imagen(tmp_path):
    """PNG sintético con bytes no-ASCII para forzar el camino difícil."""
    ruta = tmp_path / "captura.png"
    ruta.write_bytes(bytes(range(256)) * 400)  # 100 KB
    return ruta


@pytest.fixture()
def imagenes(tmp_path):
    rutas = []
    for nombre, contenido in (("a.png", b"\x89PNG" + b"\x00" * 511), ("b.jpg", b"\xff\xd8" * 90)):
        ruta = tmp_path / nombre
        ruta.write_bytes(contenido)
        rutas.append(ruta)
    return rutas


# --------------------------------------------------------------------------
# Codificación en streaming
# --------------------------------------------------------------------------
def test_b64_stream_coincide_con_b64encode_de_una_pieza(imagen):
    esperado = base64.b64encode(imagen.read_bytes())
    with imagen.open("rb") as fh:
        obtenido = vision.b64_stream(fh, chunk_size=777)  # trozos desalineados a propósito
    assert bytes(obtenido) == esperado


def test_b64_stream_no_carga_el_archivo_entero():
    """El pico de memoria debe quedar acotado por el chunk, no por el archivo."""
    fuente = io.BytesIO(b"A" * (10 * 1024 * 1024))
    leido = []

    class Espia(io.BytesIO):
        def read(self, n=-1):
            data = super().read(n)
            leido.append(len(data))
            return data

    obtenido = vision.b64_stream(Espia(b"A" * (10 * 1024 * 1024)), chunk_size=48 * 1024)
    assert len(obtenido) > 0
    assert max(leido[:-1]) <= 48 * 1024  # nunca un read() del archivo completo


def test_b64_stream_respeta_el_limite():
    with pytest.raises(vision.VisionError):
        vision.b64_stream(io.BytesIO(b"A" * 5000), chunk_size=1024, max_bytes=1000)


def test_encode_file_b64_una_sola_lectura(imagen, monkeypatch):
    aperturas = []
    original = Path.open

    def espia(self, *args, **kwargs):
        aperturas.append(str(self))
        return original(self, *args, **kwargs)

    monkeypatch.setattr(Path, "open", espia)
    vision.encode_file_b64(imagen)
    assert len(aperturas) == 1


def test_encode_file_inexistente_lanza_visionerror(tmp_path):
    with pytest.raises(vision.VisionError):
        vision.encode_file_b64(tmp_path / "no.png")


def test_archivo_vacio_lanza_visionerror(tmp_path):
    vacio = tmp_path / "vacio.png"
    vacio.write_bytes(b"")
    with pytest.raises(vision.VisionError):
        vision.pack_inline_data(vacio)


# --------------------------------------------------------------------------
# MIME
# --------------------------------------------------------------------------
@pytest.mark.parametrize(
    "nombre,esperado",
    [
        ("a.png", "image/png"),
        ("a.PNG", "image/png"),
        ("a.jpg", "image/jpeg"),
        ("a.jpeg", "image/jpeg"),
        ("a.webp", "image/webp"),
        ("a.wav", "audio/wav"),
        ("a.desconocido", "image/png"),
    ],
)
def test_guess_mime(nombre, esperado):
    assert vision.guess_mime(nombre) == esperado


# --------------------------------------------------------------------------
# Empaquetado inline_data
# --------------------------------------------------------------------------
def test_pack_inline_data_roundtrip_identico(imagen):
    part = vision.pack_inline_data(imagen)
    assert part["inlineData"]["mimeType"] == "image/png"
    assert base64.b64decode(part["inlineData"]["data"]) == imagen.read_bytes()


def test_pack_inline_data_reutiliza_data_sin_releer(imagen, monkeypatch):
    data = base64.b64encode(imagen.read_bytes())  # se lee ANTES de instrumentar

    aperturas = []
    original = Path.open
    monkeypatch.setattr(
        Path, "open", lambda self, *a, **k: (aperturas.append(str(self)), original(self, *a, **k))[1]
    )
    part = vision.pack_inline_data(imagen, data=data)
    assert aperturas == []  # cero lecturas de disco
    assert base64.b64decode(part["inlineData"]["data"]) == imagen.read_bytes()


def test_vision_contents_estructura_gemini(imagenes):
    contents = vision.vision_contents("¿qué ves?", imagenes)
    assert len(contents) == 1
    assert contents[0]["role"] == "user"
    parts = contents[0]["parts"]
    assert parts[0] == {"text": "¿qué ves?"}
    assert len(parts) == 1 + len(imagenes)
    assert all("inlineData" in p for p in parts[1:])


# --------------------------------------------------------------------------
# Payload de una sola pasada (el núcleo de la optimización)
# --------------------------------------------------------------------------
def test_build_payload_bytes_es_json_valido_y_fiel(imagenes):
    payload = vision.build_payload_bytes("describe con acentos áé 🚀", imagenes)
    cuerpo = json.loads(payload)
    parts = cuerpo["contents"][0]["parts"]

    assert parts[0]["text"] == "describe con acentos áé 🚀"
    for ruta, part in zip(imagenes, parts[1:]):
        assert base64.b64decode(part["inlineData"]["data"]) == ruta.read_bytes()


def test_build_payload_bytes_no_deja_centinela(imagenes):
    payload = vision.build_payload_bytes("x", imagenes)
    assert b"NXB64" not in payload
    assert b"\\u0000" not in payload


def test_build_payload_bytes_lee_cada_archivo_una_sola_vez(imagenes, monkeypatch):
    aperturas = []
    original = Path.open
    monkeypatch.setattr(
        Path, "open", lambda self, *a, **k: (aperturas.append(str(self)), original(self, *a, **k))[1]
    )
    vision.build_payload_bytes("x", imagenes)
    assert len(aperturas) == len(imagenes)


def test_build_payload_bytes_nunca_escribe_temporales(imagenes, tmp_path, monkeypatch):
    """No debe existir ningún archivo intermedio en disco."""
    import tempfile

    def prohibido(*args, **kwargs):
        raise AssertionError("build_payload_bytes no debe crear temporales")

    monkeypatch.setattr(tempfile, "mkstemp", prohibido)
    monkeypatch.setattr(tempfile, "NamedTemporaryFile", prohibido)

    antes = set(os.listdir(tmp_path))
    vision.build_payload_bytes("x", imagenes)
    assert set(os.listdir(tmp_path)) == antes


def test_build_payload_bytes_sin_escritura_en_disco(tmp_path, monkeypatch):
    escrituras = []
    original = Path.write_bytes
    monkeypatch.setattr(
        Path, "write_bytes", lambda self, data: escrituras.append(str(self))
    )
    ruta = tmp_path / "x.png"
    original(ruta, b"\x89PNG" + b"\x00" * 100)
    escrituras.clear()

    vision.build_payload_bytes("x", [ruta])
    assert escrituras == []


def test_build_payload_bytes_acota_el_tamano(imagenes):
    with pytest.raises(vision.VisionError):
        vision.build_payload_bytes("x", imagenes, max_bytes=10)


def test_build_payload_bytes_rechaza_archivo_inexistente(tmp_path):
    with pytest.raises(vision.VisionError):
        vision.build_payload_bytes("x", [tmp_path / "no.png"])


def test_build_payload_bytes_sin_imagenes_falla(tmp_path):
    with pytest.raises(vision.VisionError):
        vision.build_payload_bytes("x", [])


def test_build_payload_bytes_incluye_generation_config(imagenes):
    payload = vision.build_payload_bytes(
        "x", imagenes, generation_config={"temperature": 0.9}
    )
    assert json.loads(payload)["generationConfig"] == {"temperature": 0.9}


def test_payload_es_bytes_no_str(imagenes):
    """Devolver bytes evita el .encode() posterior que duplicaría el buffer."""
    assert isinstance(vision.build_payload_bytes("x", imagenes), bytes)


# --------------------------------------------------------------------------
# Degradación
# --------------------------------------------------------------------------
def test_describe_image_for_text_models_incluye_las_rutas(imagenes):
    texto = vision.describe_image_for_text_models(imagenes)
    assert str(imagenes[0]) in texto
    assert "inline_data" in texto


def test_describe_image_sin_imagenes():
    assert "(ninguna)" in vision.describe_image_for_text_models([])
