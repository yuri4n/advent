#!/usr/bin/env python3
"""Small real stdio LSP peer for startup/ownership/diagnostic regression tests."""

import argparse
import json
import os
import sys
import time

parser = argparse.ArgumentParser()
parser.add_argument("--delay", type=float, default=0)
parser.add_argument("--tag", default="fallback")
args = parser.parse_args()


def send(message):
    payload = json.dumps(message).encode()
    sys.stdout.buffer.write(f"Content-Length: {len(payload)}\r\n\r\n".encode() + payload)
    sys.stdout.buffer.flush()


while True:
    headers = {}
    while line := sys.stdin.buffer.readline():
        if line in (b"\r\n", b"\n"):
            break
        key, value = line.decode().split(":", 1)
        headers[key.lower()] = value.strip()
    if not line:
        break
    message = json.loads(sys.stdin.buffer.read(int(headers["content-length"])))
    method = message.get("method")
    with open(os.environ["AOC_TEST_LSP_LOG"], "a", encoding="utf-8") as log:
        log.write(json.dumps({"method": method, "tag": args.tag, "pid": os.getpid()}) + "\n")
    if method == "exit":
        break
    if method == "initialize":
        time.sleep(args.delay)
        result = {
            "capabilities": {
                "textDocumentSync": 1,
                "hoverProvider": True,
                "completionProvider": {"triggerCharacters": ["."]},
            }
        }
    elif method == "textDocument/hover":
        result = {"contents": {"kind": "plaintext", "value": "Fixture hover"}}
    elif method == "textDocument/completion":
        result = []
    else:
        result = None
    if "id" in message:
        send({"jsonrpc": "2.0", "id": message["id"], "result": result})
    if method == "textDocument/didOpen":
        document = message["params"]["textDocument"]
        diagnostics = []
        if "type_error" in document["text"]:
            diagnostics.append({
                "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 4}},
                "severity": 1,
                "source": "advent-fixture",
                "message": "Intentional type error from fixture language server",
            })
        send({"jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": {
            "uri": document["uri"], "diagnostics": diagnostics,
        }})
