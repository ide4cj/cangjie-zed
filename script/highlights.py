#!/usr/bin/env python3
"""languages/cangjie/highlights.scm from tree-sitter-cangjie's queries/highlights.scm.

Zed reads queries only from the extension: this copies the grammar's, captures as the grammar names
them (cjls's D105). A theme without a name falls back along its dots, and leaves a capture none of
whose prefixes it has, `@spell` among them, uncolored.

    script/highlights.py <tree-sitter-cangjie checkout>           # write it
    script/highlights.py <tree-sitter-cangjie checkout> --check   # fail if it differs
"""
import sys
from pathlib import Path

HEADER = """\
; Copied by script/highlights.py from tree-sitter-cangjie's queries/highlights.scm (MIT, BonZer0)
; at the revision extension.toml pins. Don't edit: move the pin and run the script.

"""


def main() -> None:
    args = [a for a in sys.argv[1:] if a != "--check"]
    if len(args) != 1:
        sys.exit(__doc__)
    here = Path(__file__).resolve().parent
    upstream = Path(args[0], "queries", "highlights.scm").read_text(encoding="utf-8")
    generated = HEADER + upstream
    target = here.parent / "languages" / "cangjie" / "highlights.scm"
    if "--check" in sys.argv:
        if target.read_text(encoding="utf-8") != generated:
            sys.exit(f"{target.relative_to(here.parent)} is not what script/highlights.py makes: run it")
    else:
        target.write_text(generated, encoding="utf-8")


if __name__ == "__main__":
    main()
