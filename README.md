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

## Settings

The server's settings go in `lsp.cjls.settings`, beside the extension's own `version`:

```json
{
  "lsp": {
    "cjls": {
      "settings": { "linkedProjects": ["/path/to/cj-project.json", "/path/to/dir"] }
    }
  }
}
```

cjls finds a project under each folder Zed opens: `cj-project.json`, else `cjpm.toml`, else the
loose `.cj` files; `linkedProjects` adds others. The extension gives the server these settings as
its `initialization_options` (over which Zed merges `lsp.cjls.initialization_options`, if set) and
as the section `cjls` of its configuration; a cjls that prints its settings' schema asks for that
section again whenever Zed says the settings changed, without a restart ([cjls's D79](https://github.com/ide4cj/cjls/blob/master/docs/adr/0079-settings-are-r1s.md)).

What cjls reads, as `cjls --config-schema` describes it (CI holds this table to the cjls the
extension pins; a key a cjls does not know, `version` among them, it leaves alone):

<!-- settings: made by script/settings.py from `cjls --config-schema` -->
| `lsp.cjls.settings.` | type | default | |
|---|---|---|---|
| `linkedProjects` | string[] | `[]` | Projects to load, as rust-analyzer's: each an absolute path or a `file:` URI of a `cj-project.json`, or of a directory to find a project in. Under a root without a `cj-project.json`, they replace finding one there; outside every root, they are loaded besides. |
| `cangjieHome` | string or null | `null` | The Cangjie SDK, an absolute path or a `file:` URI: looked for first, before `CANGJIE_HOME`, cjsdk's default toolchain and `cjc` on `PATH`. `std` is taken from it, and `${CANGJIE_HOME}` in a project's paths expands to it. |
| `cangjieSrc` | string or null | `null` | The sources of the SDK's version, a directory holding `std/` as `cangjie_runtime`'s `stdlib/libs` does, an absolute path or a `file:` URI: looked for first, before `CANGJIE_SRC` and the SDK's `src`. Go to definition on a declaration of `std` goes there; `cjls fetch-src` puts them in the SDK's `src`. |
<!-- /settings -->

## Highlighting

Tree-sitter colors a file as it is typed; what the tree cannot tell — a call of a type or of an enum
constructor, a name at its use — cjls's semantic tokens tell, laid over it once Zed asks for them
(off by default, a setting of the user's):

```json
{
  "languages": {
    "Cangjie": { "semantic_tokens": "combined" }
  }
}
```

The extension follows the grammar's names and renames none: `highlights.scm` is the grammar's own,
and the server sends no token the grammar colors as well, naming the others in the grammar's
classes ([cjls's D105](https://github.com/ide4cj/cjls/blob/master/docs/adr/0105-semantic-tokens-augment-the-grammar.md)).
`languages/cangjie/semantic_token_rules.json` gives each type and modifier of the server's the
capture tree-sitter gives the same place, then that capture's prefixes, as a theme falls back for
tree-sitter. `global_lsp_settings.semantic_token_rules` in the settings goes before them.

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
script/settings.py <cjls>                     # the server's settings into README.md
test/queries.sh <tree-sitter-cangjie checkout built with `tree-sitter build`>
test/smoke.py <cjls> test                     # cjls started as the extension starts it, with settings
```

`languages/cangjie/highlights.scm` is generated: tree-sitter-cangjie's `queries/highlights.scm` at the
revision `extension.toml` pins, as it is. To take the grammar's changes, move `rev` (Renovate proposes
its `master` weekly) and run the script; CI fails while the file is not what the script makes.

A change the server has to make first is a branch of the same name here and in cjls (cjls's D32,
[CONTRIBUTING](https://github.com/ide4cj/.github/blob/master/CONTRIBUTING.md)). A release (`bump.yml`,
by hand, or when Renovate moves `.cjls-version` to a new minor) opens the PR to
[zed-industries/extensions](https://github.com/zed-industries/extensions) from the fork
`ide4cj/extensions` when `ZED_EXTENSIONS_TOKEN` is set.

## License

MIT or Apache-2.0, as cjls. `languages/cangjie/highlights.scm` is made from tree-sitter-cangjie's
(MIT).
