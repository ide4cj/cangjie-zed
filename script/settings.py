#!/usr/bin/env python3
"""The server's settings in README.md, from the JSON Schema `cjls --config-schema` prints (cjls's
D32, D79): a row per setting between the markers, its type, default and description. Zed takes no
schema from an extension of zed_extension_api 0.7, so the README is where they are listed.

    script/settings.py <cjls>            # write it
    script/settings.py <cjls> --check    # fail if it differs

A cjls before its settings had a schema prints none: the check then passes, with a notice.
"""
import json
import subprocess
import sys
from pathlib import Path

README = Path(__file__).resolve().parent.parent / "README.md"
BEGIN = "<!-- settings: made by script/settings.py from `cjls --config-schema` -->"
END = "<!-- /settings -->"


def type_of(schema):
    types = schema["type"] if isinstance(schema["type"], list) else [schema["type"]]
    return " or ".join(f"{schema['items']['type']}[]" if t == "array" and "items" in schema else t
                       for t in types)


def rows(schema):
    yield "| `lsp.cjls.settings.` | type | default | |"
    yield "|---|---|---|---|"
    for key, p in schema["properties"].items():
        description = p.get("description", "").replace("\n", " ")
        yield f"| `{key}` | {type_of(p)} | `{json.dumps(p.get('default'))}` | {description} |"


def main(cjls, check=False):
    run = subprocess.run([cjls, "--config-schema"], capture_output=True, text=True)
    if run.returncode != 0:
        if "no command `--config-schema`" in run.stderr:
            print(f"::notice::{cjls} prints no settings' schema: README.md is not checked against it")
            return 0
        sys.exit(f"{cjls} --config-schema failed: {run.stderr}")
    text = README.read_text(encoding="utf-8")
    start, end = text.index(BEGIN) + len(BEGIN), text.index(END)
    made = text[:start] + "\n" + "\n".join(rows(json.loads(run.stdout))) + "\n" + text[end:]
    if not check:
        README.write_text(made, encoding="utf-8")
    elif made != text:
        print(f"::error::README.md's settings are not those of {cjls}: run script/settings.py {cjls}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], check="--check" in sys.argv[2:]))
