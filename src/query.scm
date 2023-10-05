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
 (address4)
 (address6)
] @leaf

(
  [
    (module_decl)
    (import)
  ] @append_hardline
  .
  (comment)? @do_nothing
)

(import "import" @append_space)
(module_decl "module" @append_space)

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
  (unit_switch)
 ] @append_empty_softline
  .
  (comment)? @do_nothing
)

("struct"
 [
  (field_decl)
 ] @append_empty_softline
  .
  (comment)? @do_nothing
)

(struct_decl "struct" @append_space)

(unit_switch
  "switch" @append_space
)
(unit_switch
  (unit_switch_case) @append_hardline
  .
  (comment)? @do_nothing
)
(unit_switch_case
  "->" @prepend_space @append_space
)

(switch
  "switch" @append_space
  "(" (expression) ")" @append_space
  [
    (case "case" @append_space)
    (comment)
  ]* @append_spaced_softline
)

(case
  ":" @append_indent_start
  (comment)*
  ; We only match statements with a single expression here since statements
  ; with blocks do their own indention.
  (statement (expression)) @append_indent_end
)

; Switch with local binding.
(switch
  (linkage) @append_space
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
 "type"
 "enum"
 "unit"
 "on"
 "="
 (inout)
 (is_debug)
 "if"
 (hook_priority)
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
   "&"
   "|"
  ] @prepend_space @append_space
  (_)
)

(assign
  (expression) @append_space
  ; This automatically inserts spaces around the operator.
  (expression) @prepend_space
)

; Disambiguate negative numbers.
(unary_op "-" @append_antispace (_))

[
 ","
 "new"
] @append_space

(optional)@prepend_space

(assert
  (_) @prepend_space
  (":" @prepend_space @append_space (_))?
)

[
 ":"
] @append_space

[
 (comment)
 (type_decl)
 (field_decl)
 (unit_switch)
 (struct_decl)
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
 (function_decl)
]
 @append_hardline

[
 (field_decl)
 (sink_decl)
] @append_input_softline

(visibility) @append_space

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

(function_decl "function" @append_space)

(
 [
  (var_decl)
 ] @append_hardline @prepend_hardline
 .
 (comment)? @do_nothing
)

(var_decl
  (linkage) @append_space
)

(sink_decl "sink" @append_space)

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

(print "print" @append_space)
(return "return" @append_space)
(throw_ "throw" @append_space)
(unset "unset" @append_space)

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
  "for" @prepend_space
  "(" @prepend_space
  (_) @append_space
  "in" @append_space
  (_)
  ")" @append_space
)
(for
  ")" @append_indent_start
  (statement (block)*@do_nothing) @append_indent_end
)

(while
  "while" @prepend_space
  "(" @prepend_space
  ")" @append_space
)

(list_comp
  "for" @prepend_space @append_space
  "in" @prepend_space @append_space
)

(contains
  "in" @prepend_space @append_space
)
(contains_not
  "!in" @prepend_space @append_space
)

(foreach "foreach" @prepend_space @append_space)

(function_call
  (ident)
  "(" @append_hardline @append_indent_start
  ((expression) . "," @append_hardline)+
  ")" @prepend_indent_end @prepend_hardline
  (#multi_line_only!)
)

; Suppress space before field `;` decl with `if.
(field_decl
  "skip"? @append_space
  (if (statement ";") @prepend_antispace)?
)
; Enforce spaces around `->` in field sink syntax.
(field_decl
  "->" @prepend_space @append_space
  (sink)
)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Bitfields.

(
 "bitfield"
 (_)
 "{" @prepend_space
 "}" @prepend_hardline
)

; All bitfield fields go on a new line.
(bitfield
  (bitfield_field) @prepend_hardline
)

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Preprocessor macros.

; Empty lines before preprocessor blocks are fine.
(preproc) @allow_blank_line_before

; `@if` always has a following expression so it needs a space.
(preproc "@if" @append_space)

; Newline handling. This is slightly nasty since preprocessor macros are
; terminated by explicit newlines instead of `;`. Force a newline, but delete
; original newlines so we do not insert stray empty lines or spaces.
(preproc
  "@if" (expression) @append_hardline
  "\n" @delete
)
(preproc
  "@else" @append_hardline
  "\n" @delete
)
(preproc
  "@endif" @prepend_hardline
)
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
