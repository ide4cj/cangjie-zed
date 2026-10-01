; ===== Zed =====
; Zed's themes color brackets and delimiters apart from operators; the grammar's queries capture
; them all as @operator.

["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["." "," ":" ";" "::"] @punctuation.delimiter

; The unit () again after its brackets, as the grammar's queries place it after @operator.
(unit_literal) @constant.builtin
