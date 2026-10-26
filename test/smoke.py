#!/usr/bin/env python3
"""cjls as the extension launches it: no arguments, stdio, a folder for its root. Initialized,
shut down and exited, it must answer with its name and exit 0. Then its settings as Zed passes
those of `lsp.cjls.settings` (src/lib.rs, Zed's `lsp_store.rs`): as `initializationOptions`, in
`workspace/didChangeConfiguration` after `initialized`, and in the answers to
`workspace/configuration`, the section `cjls` of what the extension gives; a project linked there
must be loaded. Zed runs no extension headless: what this cannot see is cjls's tests/e2e (cjls's
D32).

    test/smoke.py <cjls> <folder>
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

CONTENT_MODIFIED = -32801


def send(server, message):
    body = json.dumps({"jsonrpc": "2.0", **message}).encode()
    server.stdin.write(b"Content-Length: %d\r\n\r\n" % len(body) + body)
    server.stdin.flush()


def receive(server, id, configuration=None):
    """The response to `id`, skipping the server's notifications on the way and answering its
    requests: `workspace/configuration` from `configuration` as Zed does, the others with `null`."""
    while True:
        length = None
        while (line := server.stdout.readline()) != b"\r\n":
            if not line:
                raise SystemExit("cjls closed stdout")
            name, _, value = line.partition(b":")
            if name.lower() == b"content-length":
                length = int(value)
        message = json.loads(server.stdout.read(length))
        if "method" in message and "id" in message:
            result = None
            if message["method"] == "workspace/configuration":
                config = configuration or {}
                result = [config.get(item["section"]) if "section" in item else config
                          for item in message["params"]["items"]]
            send(server, {"id": message["id"], "result": result})
        elif message.get("id") == id:
            return message


def main(cjls, folder):
    # arrange
    root = Path(folder).resolve()
    server = subprocess.Popen([cjls], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    try:
        # act
        send(server, {"id": 1, "method": "initialize", "params": {
            "processId": None,
            "rootUri": root.as_uri(),
            "workspaceFolders": [{"uri": root.as_uri(), "name": root.name}],
            "capabilities": {},
        }})
        initialized = receive(server, 1)

        # assert
        assert "result" in initialized, initialized
        name = initialized["result"].get("serverInfo", {}).get("name")
        assert name == "cjls", initialized["result"]

        # act
        send(server, {"method": "initialized", "params": {}})
        send(server, {"id": 2, "method": "shutdown"})
        shutdown = receive(server, 2)

        # assert
        assert "result" in shutdown, shutdown

        # act
        send(server, {"method": "exit"})
        code = server.wait()

        # assert
        assert code == 0, f"cjls exited with {code}"
    finally:
        if server.poll() is None:
            server.kill()
            server.wait()
    print(f"{name} {initialized['result']['serverInfo'].get('version')}: initialized, shut down, exited")


def settings(cjls):
    """A project linked by `lsp.cjls.settings.linkedProjects` is loaded: its symbol is found."""
    # arrange
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp, "ws")
        root.mkdir()
        linked = Path(tmp, "linked")
        linked.mkdir()
        (linked / "linked.cj").write_text("func findLinked() {}\n")
        # what src/lib.rs gives Zed from `lsp.cjls.settings`, the extension's own `version` among them
        user = {"linkedProjects": [str(linked)], "version": "nightly-build"}
        configuration = {"cjls": user}
        server = subprocess.Popen([cjls], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        try:
            # act
            send(server, {"id": 1, "method": "initialize", "params": {
                "processId": None,
                "rootUri": root.as_uri(),
                "workspaceFolders": [{"uri": root.as_uri(), "name": root.name}],
                "capabilities": {"workspace": {
                    "configuration": True,
                    "didChangeConfiguration": {"dynamicRegistration": True},
                }},
                "initializationOptions": user,
            }})
            initialized = receive(server, 1, configuration)

            # assert
            assert "result" in initialized, initialized

            # act: linked projects load in the background
            send(server, {"method": "initialized", "params": {}})
            send(server, {"method": "workspace/didChangeConfiguration",
                          "params": {"settings": configuration}})
            # loaded in the background, answering nothing or ContentModified meanwhile: asked until
            # it has, a hang is CI's to end
            id = 2
            while True:
                # act
                send(server, {"id": id, "method": "workspace/symbol", "params": {"query": "findLinked"}})
                answer = receive(server, id, configuration)
                id += 1
                error = answer.get("error")

                # assert: a loading snapshot may ask the client to retry
                assert error is None or error["code"] == CONTENT_MODIFIED, answer
                if error is None and answer["result"]:
                    break
            names = [s["name"] for s in answer["result"]]

            # assert
            assert names == ["findLinked"], names

            # act
            send(server, {"id": id, "method": "shutdown"})
            shutdown = receive(server, id, configuration)

            # assert
            assert "result" in shutdown, shutdown

            # act
            send(server, {"method": "exit"})
            code = server.wait()

            # assert
            assert code == 0, f"cjls exited with {code}"
        finally:
            if server.poll() is None:
                server.kill()
                server.wait()
    print(f"lsp.cjls.settings: {linked} linked, its symbol found")


if __name__ == "__main__":
    main(*sys.argv[1:])
    settings(sys.argv[1])
