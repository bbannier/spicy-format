; Configuration
; topiary hardcodes the list of known languages, fool it to think it knows spicy.
; (#language! spicy)
(#language! json)
(#indent! "    ")

; It is currently impossible to format qualified identifiers, see
; https://github.com/tweag/topiary/issues/418.
[
 (attribute_name)
 (ident)
 (integer)
] @leaf

(
  (module_decl) @append_hardline
  .
  (comment)? @do_nothing
)

; If we have anything (e.g., comments) before the initial module decl, preserve empty lines after it.
(
  (_)
  .
  (module_decl) @allow_blank_line_before
)

("unit"
 [
  (field_decl)
  (hook_decl)
  (sink_decl)
  (unit_switch
    "switch" @append_space
   (unit_switch_case) @append_hardline
   .
   (comment)? @do_nothing
  )
 ] @append_hardline
  .
  (comment)? @do_nothing
)

(switch
  "switch" @append_space
  "(" (expression) ")" @append_space
  "{" @append_indent_start

  (case
    "case" @append_space
    ":" @append_indent_start
    )* @prepend_indent_end @append_spaced_softline
  "}" @prepend_indent_end
)

(
 (enum_label) @append_delimiter
 .
 ","? @delete
 (#delimiter! ",")
)

("{" @append_hardline
 (enum_label)
 .
 ((enum_label))+
 "}" @prepend_hardline
)
(
 (enum_label)
 .
 (enum_label) @prepend_hardline
)

[
 "import"
 "module"
 "public"
 "type"
 "enum"
 "unit"
 "unset"
 "sink"
 "var"
 "print"
 "on"
 "function"
 "return"
 "global"
 "local"
 "const"
 "for"
 "foreach"
 "in"
 "->"
 "="
 "+="
 "-="
 "*="
 "/="
 (inout)
 (is_debug)
 "if"
 (hook_priority)
 "skip"
] @append_space @prepend_space

(binary_op
  (_)
  [
   "+"
   "-"
   "*"
   "/"
   "%"
   ">"
   "<"
   ">="
   "<="
   "=="
   "!="
   "&&"
   "||"
  ] @prepend_space @append_space
  (_)
)

; Disambiguate negative numbers.
(unary_op "-" @append_antispace (_))

[
 ","
 "new"
] @append_space

(optional)@prepend_space

(assert (_)@prepend_space)

[
 ":"
] @append_space

[
 (comment)
 (type_decl)
 (field_decl)
 (unit_switch)
 (sink_decl)
 (hook_decl)
 (function_decl)
 (var_decl)
 (enum_decl)
 (attribute)
 (statement)
 (import)
] @allow_blank_line_before

[
 (enum_decl)
 (type_decl)
 (import)
]
 @append_hardline

[
 (field_decl)
 (sink_decl)
] @append_input_softline

(comment) @prepend_input_softline @append_input_softline @append_hardline

(attribute) @prepend_space
(attribute (_) @append_antispace "=" @append_antispace (_))

[(block)] @prepend_space

("{") @append_spaced_softline @append_indent_start
("}") @prepend_spaced_softline @prepend_indent_end
(
 "{"
 .
 "}" @prepend_antispace
)

(
 ";" @append_spaced_softline
 .
 (comment)? @do_nothing
)

(
 [
  (var_decl)
 ] @append_hardline @prepend_hardline
 .
 (comment)? @do_nothing
)

(statement) @prepend_input_softline

(
 (
  (statement)
  .
  (comment)? @do_nothing
 ) @prepend_hardline @append_hardline
 .
 (
  (statement) . (comment)? @do_nothing
 ) @append_hardline
)

(statement (_) ";" @prepend_antispace)

(throw_ "throw" @append_space)

(list
 "," @append_spaced_softline
)

(ternary
  (_)
  "?" @prepend_space @append_space
  (_)
  ":" @prepend_space @append_space
)

("unit"
  (params ((_)+)) @prepend_antispace @append_space
)

; Remove empty `()` in `unit ()`.
("unit"
  (params
    ((ident) @do_nothing)*
  ) @delete
)

; Unit `switch` statement.
(unit_switch (expression) ")" @append_space)

(if
  (expression) @append_indent_start
  (statement (block)*@do_nothing) @prepend_input_softline @append_indent_end
)

(for
  ")" @append_indent_start
  (statement (block)*@do_nothing) @append_indent_end
)

(function_call
  (ident)
  "(" @append_hardline @append_indent_start
  ((expression) . "," @append_hardline)+
  ")" @prepend_indent_end @prepend_hardline
  (#multi_line_only!)
)

; Suppress space before field `;` decl with `if.
(field_decl
  (if (statement ";") @prepend_antispace)
)

(
 "bitfield"
 (_)
 "{" @prepend_space
)
