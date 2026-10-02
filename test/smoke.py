#!/usr/bin/env python3
"""cjls as the extension launches it: no arguments, stdio, a folder for its root. Initialized,
shut down and exited, it must answer with its name and exit 0. Zed runs no extension headless:
what this cannot see is cjls's tests/e2e (cjls's D32).

    test/smoke.py <cjls> <folder>
"""

import json
import subprocess
import sys
from pathlib import Path


def send(server, message):
    body = json.dumps({"jsonrpc": "2.0", **message}).encode()
    server.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    server.stdin.flush()


def receive(server, id):
    """The response to `id`, skipping the server's notifications and requests on the way."""
    while True:
        length = None
        while (line := server.stdout.readline()) != b"\r\n":
            if not line:
                raise SystemExit("cjls closed stdout")
            name, _, value = line.partition(b":")
            if name.lower() == b"content-length":
                length = int(value)
        message = json.loads(server.stdout.read(length))
        if message.get("id") == id and "method" not in message:
            return message


def main(cjls, folder):
    root = Path(folder).resolve()
    server = subprocess.Popen([cjls], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        send(server, {"id": 1, "method": "initialize", "params": {
            "processId": None,
            "rootUri": root.as_uri(),
            "workspaceFolders": [{"uri": root.as_uri(), "name": root.name}],
            "capabilities": {},
        }})
        initialized = receive(server, 1)
        assert "result" in initialized, initialized
        name = initialized["result"].get("serverInfo", {}).get("name")
        assert name == "cjls", initialized["result"]
        send(server, {"method": "initialized", "params": {}})
        send(server, {"id": 2, "method": "shutdown"})
        assert "result" in receive(server, 2)
        send(server, {"method": "exit"})
        code = server.wait(timeout=60)
        assert code == 0, f"cjls exited with {code}"
    finally:
        if server.poll() is None:
            server.kill()
    print(f"{name} {initialized['result']['serverInfo'].get('version')}: initialized, shut down, exited")


if __name__ == "__main__":
    main(*sys.argv[1:])
