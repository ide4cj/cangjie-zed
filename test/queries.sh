#!/usr/bin/env bash
# The queries against the grammar extension.toml pins: test/sample.cj parses without an error, and
# every query compiles and matches in it (a node or field the grammar lacks fails the query).
#
#   test/queries.sh <tree-sitter-cangjie checkout, built with `tree-sitter build`>
set -euo pipefail

# arrange
grammar=$1
here=$(cd "$(dirname "$0")/.." && pwd)
sample="$here/test/sample.cj"

# act
parsed=$(tree-sitter parse --grammar-path "$grammar" "$sample")

# assert
if [[ "$parsed" == *ERROR* ]]; then
  echo "test/sample.cj does not parse:"
  echo "$parsed"
  exit 1
fi
for q in "$here"/languages/cangjie/*.scm; do
  # act
  matches=$(tree-sitter query --grammar-path "$grammar" "$q" "$sample")
  captures=$(grep -c 'capture:' <<< "$matches" || true)

  # assert
  echo "$(basename "$q"): $captures captures"
  if [ "$captures" = 0 ]; then
    echo "$matches"
    exit 1
  fi
done
