"""
nexo_cache — Historial de conversación recortado en memoria con persistencia atómica.

Objetivo: reducir el overhead del ciclo de memoria del agente en dispositivos
aarch64 con menos de 1 GB de RAM libre.

Antes: ``json.load`` del fichero completo -> ``mem[-8:]`` -> ``json.dump(..., indent=2)``
en cada turno. Con ``indent=2`` el fichero crece ~35% y se re-serializa entero.

Ahora:
  * El recorte se aplica ANTES de persistir, así que el fichero nunca crece sin
    límite y la deserialización de arranque es mínima.
  * Salida compacta (sin ``indent``) => menos bytes escritos en la eMMC/SD.
  * Escritura atómica (temporal + ``os.replace``): un corte de batería o un
    ``lowmemorykiller`` actuando a mitad de escritura no corrompe el historial.
  * Nunca se mantiene una copia ``str`` del JSON: se codifica a bytes y se
    escribe directamente.
"""

from __future__ import annotations

import json
import os
import tempfile
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

__all__ = ["TurnCache"]

Message = Dict[str, Any]


class TurnCache:
    """Historial de corto plazo con tope de turnos."""

    def __init__(
        self,
        path: Union[str, os.PathLike, None] = None,
        *,
        max_turns: int = 8,
        max_bytes: int = 512 * 1024,
    ):
        self.path: Optional[Path] = Path(path) if path else None
        self.max_turns = max(1, int(max_turns))
        self.max_bytes = max(4096, int(max_bytes))
        self._messages: List[Message] = []
        if self.path is not None:
            self._messages = self._read()

    # ------------------------------------------------------------------
    def _read(self) -> List[Message]:
        if not self.path or not self.path.exists():
            return []
        try:
            # Acotar la lectura: un historial inflado por una version anterior no
            # debe cargarse entero en un dispositivo con poca RAM.
            size = self.path.stat().st_size
            with self.path.open("rb") as fh:
                if size > self.max_bytes:
                    fh.seek(-self.max_bytes, os.SEEK_END)
                    fh.readline()  # descartar la primera linea partida
                raw = fh.read()
            data = json.loads(raw.decode("utf-8", errors="replace"))
        except (OSError, ValueError):
            return []
        if not isinstance(data, list):
            return []
        messages = [m for m in data if isinstance(m, dict) and m.get("role")]
        return self._trim(messages)

    def _trim(self, messages: List[Message]) -> List[Message]:
        limit = self.max_turns * 2  # 1 turno = user + assistant
        if len(messages) > limit:
            messages = messages[-limit:]
        return messages

    @staticmethod
    def _encode(messages: List[Message]) -> bytes:
        return json.dumps(
            messages, separators=(",", ":"), ensure_ascii=False
        ).encode("utf-8")

    @staticmethod
    def _shrink(message: Message, max_bytes: int) -> Message:
        """Último recurso: recortar el texto de un mensaje para caber en disco."""
        reducido = dict(message)
        contenido = reducido.get("content")
        if isinstance(contenido, str):
            reducido["content"] = contenido[: max(0, max_bytes // 2)] + "[...]"
        return reducido

    # ------------------------------------------------------------------
    @property
    def messages(self) -> List[Message]:
        """Vista directa (sin copia) del historial activo."""
        return self._messages

    def __len__(self) -> int:
        return len(self._messages)

    def append(self, message: Message) -> None:
        self._messages.append(message)
        self._messages = self._trim(self._messages)

    def extend(self, messages: List[Message]) -> None:
        self._messages.extend(messages)
        self._messages = self._trim(self._messages)

    def clear(self) -> None:
        self._messages = []

    # ------------------------------------------------------------------
    def save(self) -> bool:
        """Persiste el historial ya recortado. Devuelve False si no pudo."""
        if self.path is None:
            return False
        payload = self._messages
        try:
            blob = self._encode(payload)
            if len(blob) > self.max_bytes:
                # Recorte de emergencia: soltar los turnos mas antiguos hasta
                # caber, y si con un solo mensaje aun no cabe, truncar su texto.
                while len(blob) > self.max_bytes and len(payload) > 1:
                    payload = payload[1:]
                    blob = self._encode(payload)
                if len(blob) > self.max_bytes:
                    payload = [self._shrink(payload[-1], self.max_bytes)]
                    blob = self._encode(payload)
                self._messages = payload
            self.path.parent.mkdir(parents=True, exist_ok=True)
            handle, tmp_name = tempfile.mkstemp(
                dir=str(self.path.parent), prefix=".nexus_mem_", suffix=".tmp"
            )
            try:
                with os.fdopen(handle, "wb") as fh:
                    fh.write(blob)
                    fh.flush()
                    os.fsync(fh.fileno())
                os.replace(tmp_name, self.path)
            except BaseException:
                if os.path.exists(tmp_name):
                    os.unlink(tmp_name)
                raise
        except OSError:
            return False
        return True
