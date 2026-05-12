"""
Nephis Python ML Sidecar — Windows Named Pipe JSON-RPC 2.0 Server

Pipe name: \\.\pipe\NephPyside

Protocol: newline-delimited JSON-RPC 2.0
  Request:  {"jsonrpc":"2.0","id":"<uuid>","method":"<method>","params":{...}}
  Response: {"jsonrpc":"2.0","id":"<uuid>","result":{...}}
  Error:    {"jsonrpc":"2.0","id":"<uuid>","error":{"code":-32000,"message":"..."}}

Methods (Phase 1):
  stt.transcribe(audio_b64, sample_rate) -> {text, is_final}
  tts.speak(text)                        -> {audio_b64, format}  [multiple responses]
  vad.process(audio_b64, sample_rate)    -> {speech_detected}
  ping()                                 -> {pong: true}

Methods (Phase 2, stubs now):
  embed.encode(texts[])                  -> {vectors: [[float]]}
"""

import json
import sys
import traceback
import base64
import threading
import uuid
import logging
from typing import Any

import win32pipe
import win32file
import pywintypes

from nephis_pyside.stt_whisper import SttWhisper
from nephis_pyside.tts_edge import TtsEdge
from nephis_pyside.vad_silero import VadSilero

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [pyside] %(levelname)s %(message)s",
    stream=sys.stderr,
)
log = logging.getLogger("nephis_pyside")

PIPE_NAME = r"\\.\pipe\NephPyside"
PIPE_BUFFER_SIZE = 65536


class RpcServer:
    def __init__(self):
        self.stt = SttWhisper()
        self.tts = TtsEdge()
        self.vad = VadSilero()

    def dispatch(self, method: str, params: dict) -> Any:
        if method == "ping":
            return {"pong": True}
        elif method == "stt.transcribe":
            audio_bytes = base64.b64decode(params["audio_b64"])
            sample_rate = params.get("sample_rate", 16000)
            partials, text = self.stt.transcribe_with_partials(audio_bytes, sample_rate)
            return {"partials": partials, "text": text, "is_final": True}
        elif method == "tts.speak":
            text = params["text"]
            audio_bytes = self.tts.synthesize(text)
            return {"audio_b64": base64.b64encode(audio_bytes).decode(), "format": "wav"}
        elif method == "vad.process":
            audio_bytes = base64.b64decode(params["audio_b64"])
            sample_rate = params.get("sample_rate", 16000)
            detected = self.vad.process(audio_bytes, sample_rate)
            return {"speech_detected": detected}
        elif method == "embed.encode":
            # Phase 2 stub
            texts = params.get("texts", [])
            return {"vectors": [[0.0] * 384 for _ in texts], "stub": True}
        else:
            raise ValueError(f"Unknown method: {method}")


def handle_client(pipe_handle, server: RpcServer):
    """Handle one client connection (one Rust process connects at startup)."""
    buf = b""
    try:
        while True:
            try:
                _, data = win32file.ReadFile(pipe_handle, PIPE_BUFFER_SIZE)
                buf += data
            except pywintypes.error as e:
                if e.args[0] == 109:  # ERROR_BROKEN_PIPE
                    log.info("Client disconnected.")
                    break
                raise

            # Process all complete newline-delimited messages
            while b"\n" in buf:
                line, buf = buf.split(b"\n", 1)
                if not line.strip():
                    continue
                try:
                    req = json.loads(line.decode("utf-8"))
                    req_id = req.get("id")
                    method = req.get("method", "")
                    params = req.get("params", {})
                    result = server.dispatch(method, params)
                    resp = {"jsonrpc": "2.0", "id": req_id, "result": result}
                except Exception as exc:
                    log.error("RPC error: %s", traceback.format_exc())
                    resp = {
                        "jsonrpc": "2.0",
                        "id": req.get("id") if "req" in dir() else None,
                        "error": {"code": -32000, "message": str(exc)},
                    }
                out = (json.dumps(resp) + "\n").encode("utf-8")
                try:
                    win32file.WriteFile(pipe_handle, out)
                except pywintypes.error:
                    break
    finally:
        win32file.CloseHandle(pipe_handle)


def run():
    server = RpcServer()
    log.info("Nephis pyside starting on %s", PIPE_NAME)

    while True:
        pipe = win32pipe.CreateNamedPipe(
            PIPE_NAME,
            win32pipe.PIPE_ACCESS_DUPLEX,
            win32pipe.PIPE_TYPE_BYTE | win32pipe.PIPE_READMODE_BYTE | win32pipe.PIPE_WAIT,
            win32pipe.PIPE_UNLIMITED_INSTANCES,
            PIPE_BUFFER_SIZE,
            PIPE_BUFFER_SIZE,
            0,
            None,
        )
        log.info("Waiting for Rust client to connect...")
        win32pipe.ConnectNamedPipe(pipe, None)
        log.info("Client connected.")
        t = threading.Thread(target=handle_client, args=(pipe, server), daemon=True)
        t.start()


if __name__ == "__main__":
    run()
