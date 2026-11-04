# Macro-block statement grammar rewrite slice

## Source boundary

Upstream `OrcaSlicer/src/libslic3r/PlaceholderParser.cpp` processes the
inside of a single `{...}` block with a statement grammar, not only a lone
expression:

- `:2162-2164` `macros = +(block(_r1) | (statement(_r1) > (+lit(';') | &lit('}'))) | +lit(';'))`
- `:2166-2168` `block = kw["if"] > if_else_output`
- `:2157` `if_macros = kw["then"] > *(block | (statement > (+lit(';') | &(kw["elsif"] | kw["else"] | kw["endif"]))) | +lit(';'))`
- `:2159` `else_macros = *(block | (statement > (+lit(';') | &kw["endif"])))`
- `:2178-2186` `if_else_output = bool_expr > (if_text_block | if_macros) > *(kw["elsif"] > bool_expr > ...) > -(kw["else"] > ... > (if_text_block | else_macros)) > kw["endif"]`
- `:2170-2174` `statement = assignment_statement | new_variable_statement | conditional_expression` (the last emits its value through `to_string2`)
- `:2287-2300` `new_variable_statement = (kw["local"] | kw["global"]) > identifier > '=' > ...`
- `:889-906,996-1000` symbol resolution: config first, then output
  variables; `local` writes the per-invocation `config_local`, `global`
  writes `context_data->global_config`
- `:1384-1448` `new_old_variable`: a `local`/`global` whose name already
  resolves assigns that existing variable; a new name is created as an
  output variable.

Ares destination: the template layer
(`ares-core/src/project_slice/gcode_emit/template.rs` plus a new
`template/macro.rs` module). The existing recursive branch parser and
expression evaluator stay the compatibility shell; this slice adds the
missing statement grammar at the same seam, not a new pipeline.

## Diagnostic result (Native-first)

Stock profiles fail closed in Ares today: with the
verbatim `Prusa/machine/fdm_machine_common_xl_5t.json:17`
`change_filament_gcode` on a two-tool host, Native official and seed0 both
render (`G1 F21000`, `M109 S220 T1` then-arm, `T1 S1 L0 D0`) while Ares
exits 156 with `invalid expression character: ;`. Stock users of the form:
Prusa XL 5T (`:17`), Snapmaker U1 (`:11`), Anycubic Kobra X (`:44`).

## Included

- Statement lists inside one `{...}` block: statements separated by `;`,
  stray `;` allowed, trailing `;` optional before `}` and before the
  inline `elsif`/`else`/`endif` keywords (`+lit(';')` and the `&(kw[...])`
  lookaheads).
- Inline conditional blocks inside the statement list:
  `if <expr> then <statements> (elsif <expr> <statements>)* (else
  <statements>)? endif`, recursively (a nested `if` block is a statement
  list item per `block`), with only the selected arm evaluated. `then`
  is required after the condition (`if_macros`), which also keeps the
  existing text form `{if ...}...{else}...{endif}` unambiguous.
- `local NAME = <expr>` and `global NAME = <expr>` with upstream
  new/old routing: an existing variable is assigned in place, a new name
  becomes an output variable; output variables live for the remainder of
  the template render and are restored afterwards (upstream
  `config_local` lifetime).
- Expression statements emit their value (`to_string2`, `:532-537`).
- Assignments `NAME = <expr>` and `NAME[<expr>] = <expr>` reuse the
  existing `{var = expr}` machinery (`Config::assign`, zero-padded vector
  growth).

## Deferred

- Cross-invocation `global` persistence (`context_data->global_config`);
  `global` is recognized and behaves like `local` within the render. No
  stock profile uses `global` (7 profile files use `local`, none `global`).
- Initializer-list vectors `local v = (a, b)`, vector-copy assignment
  `local v = other_vector`, and `repeat(...)` array construction
  (`:2275-2299`, `:2301-2306`); no stock profile needs them.
- Mixed text-form arms inside a macro block (`if_text_block` within
  `if_macros`, i.e. `if x then ... }text{ ... endif`).
- `interpolate_table`, `switch`, statement-level ternary output nuances
  beyond the already-ported expression rules.
- Brace-containing string literals spanning placeholder boundaries (the
  existing placeholder matcher's limitation is unchanged).

## Regression contract

Freeze `tests/parity/template-macro/prusa-xl-change-filament.3mf`: the
two-tool `bbl-type1-tower/enabled.3mf` host with `change_filament_gcode`
replaced by the verbatim Prusa XL stock script, `temperature` set to
`[215, 215]` for symmetric resolution (the script's else arm reads it),
and `enable_prime_tower = 0`: the script emits no X/Y motion words, and
on a tower host the return travel then reads uninitialized upstream
state (`GCode.cpp:1010`; see the tower return travel ARD), an
upstream-UB no-witness boundary. Towerless keeps the whole stream
semantically defined.
The E2E test qualifies two ordered Native streams, asserts the rendered
toolchange block (`; Change Tool0 -> Tool1 (layer 0)`, `G1 F21000`,
`M109 S220 T1`, `T1 S1 L0 D0`) and that both tools are used, then
compares the complete public API and CLI output with the canonical
comparator, against official and seed0 replay Native.
RED must fail because Ares rejects the block at the first `;`; GREEN must
preserve whole-stream equality.
