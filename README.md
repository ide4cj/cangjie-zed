# Cangjie for Zed

[Cangjie](https://cangjie-lang.cn) in [Zed](https://zed.dev): the [cjls](https://github.com/ide4cj/cjls)
language server, downloaded on first use, and highlighting, brackets, indentation and the outline
from [tree-sitter-cangjie](https://github.com/ide4cj/tree-sitter-cangjie) (the revision
`extension.toml` pins).

## Install

**zed: extensions** → *Cangjie*. Before it is in Zed's registry: clone this repository and
**zed: install dev extension** on it (it needs Rust from rustup).

## The server binary

In this order:

1. `lsp.cjls.binary.path` in Zed's settings;
2. `cjls` on `PATH`;
3. a download from [cjls's releases](https://github.com/ide4cj/cjls/releases) (macOS arm64, Linux
   x64, Windows x64) into the extension's directory: the newest release within the minor of the one
   this extension pins in `.cjls-version`, which it is tested with, else that release itself.

```json
{
  "lsp": {
    "cjls": {
      "binary": { "path": "/path/to/cjls", "env": { "CJLS_LOG_LEVEL": "DEBUG" } },
      "settings": { "version": "nightly-build" }
    }
  }
}
```

`settings.version` downloads another release: a tag (`v0.2.0`), or `nightly-build`, cjls's master every
night, fetched again at every start of Zed. The server's log: **dev: open language server logs**.

`binary.arguments` stays empty: with none `cjls` serves LSP over stdio, and an argument it does not
know (`--stdio`) is a usage error.

## The project

cjls finds a project under each folder Zed opens: `cj-project.json`, else `cjpm.toml`, else the
loose `.cj` files. Others go in `initialization_options` ([cjls's
options](https://github.com/ide4cj/cjls/blob/master/docs/lsp-extensions.md#initialization-options)):

```json
{
  "lsp": {
    "cjls": {
      "initialization_options": { "linkedProjects": ["/path/to/cj-project.json", "/path/to/dir"] }
    }
  }
}
```

## Folding

Zed folds by tree-sitter and indentation unless told to take the server's folds, a setting of the
user's an extension cannot set. cjls folds by its own parse of the file — comments, imports, bodies,
argument lists, `match` cases:

```json
{
  "languages": {
    "Cangjie": { "document_folding_ranges": "on" }
  }
}
```

## Development

```sh
cargo test                                    # the unit tests, natively
cargo build --release --target wasm32-wasip2  # as Zed builds it
script/highlights.py <tree-sitter-cangjie checkout>  # highlights.scm from its queries
test/queries.sh <tree-sitter-cangjie checkout built with `tree-sitter build`>
test/smoke.py <cjls> test                     # cjls started as the extension starts it
```

`languages/cangjie/highlights.scm` is generated: tree-sitter-cangjie's `queries/highlights.scm` at the
revision `extension.toml` pins, its captures renamed to Zed's, then `script/highlights.zed.scm`. To
take the grammar's changes, move `rev` (Renovate proposes its `master` weekly) and run the script; CI
fails while the file is not what the script makes, and on a capture the script does not map.

A change the server has to make first is a branch of the same name here and in cjls (cjls's D32,
[CONTRIBUTING](https://github.com/ide4cj/.github/blob/master/CONTRIBUTING.md)). A release (`bump.yml`,
by hand, or when Renovate moves `.cjls-version` to a new minor) opens the PR to
[zed-industries/extensions](https://github.com/zed-industries/extensions) from the fork
`ide4cj/extensions` when `ZED_EXTENSIONS_TOKEN` is set.

## License

MIT or Apache-2.0, as cjls. `languages/cangjie/highlights.scm` is made from tree-sitter-cangjie's
(MIT).
