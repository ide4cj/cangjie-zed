#!/usr/bin/env bash
# The queries against the grammar extension.toml pins: test/sample.cj parses without an error, and
# every query compiles and matches in it (a node or field the grammar lacks fails the query).
#
#   test/queries.sh <tree-sitter-cangjie checkout, built with `tree-sitter build`>
set -euo pipefail
grammar=$1
here=$(cd "$(dirname "$0")/.." && pwd)
sample="$here/test/sample.cj"

if tree-sitter parse --grammar-path "$grammar" "$sample" 2>/dev/null | grep -q ERROR; then
  echo "test/sample.cj does not parse:"
  tree-sitter parse --grammar-path "$grammar" "$sample" | grep ERROR
  exit 1
fi
for q in "$here"/languages/cangjie/*.scm; do
  captures=$(tree-sitter query --grammar-path "$grammar" "$q" "$sample" 2>/dev/null | grep -c 'capture:' || true)
  echo "$(basename "$q"): $captures captures"
  if [ "$captures" = 0 ]; then
    tree-sitter query --grammar-path "$grammar" "$q" "$sample"
    exit 1
  fi
done
