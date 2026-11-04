# ARD: automatic brim ear runtime boundary

## Evidence and non-negotiable correction

Fresh RED44f100950 on main d7dda: E0/E1 and ear sweep fail, four non-ear
controls pass. Complete evidence is retained externally; no output/assertion filtering.

The observed validated project API goes through
`project_slice/gcode_emit/orchestration.rs:20` → `brim::BrimPlan::generate`.
`project_slice/gcode_emit/brim.rs:29` currently accepts only OuterOnly and
returns None for BrimEars. The square-ear code in `brims/ears.rs` belongs to the
separate public generate_brims scaffold; it is not the proven runtime cause of
this E2E RED. Earlier square-versus-circle observations describe a source gap,
not a demonstrated runtime call stack. Do not label the two as identical.

## Precise destination amendment

The auto-only Brim.cpp:310-352/448-460/529-573 rewrite requires:
1. A private, reusable scaled automatic ear polygon producer under
   `brims/ears.rs` (split candidate/area/path helpers only below400LOC).
2. `project_slice/gcode_emit/brim.rs::BrimPlan::generate` dispatch for
   ProcessBrimType::BrimEars, calling that upstream-owned producer with the
   same source footprint, flow spacing, quantized width and object gap used
   by the existing outer-brim path.
3. Reuse the existing BrimPlan simplification/offset/union/order/emission
   port of Brim.cpp:813-856; do not design a second ear G-code emitter.
4. Replace the square implementation in the old public adapter using the
   same upstream geometry. Do not leave square fallback or invent automatic
   points for painted mode. Painted stays entirely deferred.

OuterOnly behavior must remain byte-identical under the admitted controls.
No global scheduler, new core API, file/process access or unrelated brim
refactor is needed. ExPolygon geometry, clipping and integer coordinate
rounding must stay source-owned. E0/E1 RED and six-case sweep stay intact.

The original spec named brims/ears.rs etc but did not identify this
actual project runtime dispatch. This destination
amendment is accepted: GREEN must use a shared auto kernel from both adapters, replacing
the square implementation entirely; other brim modes are not expanded. Native/admission and source audits
must distinguish historical and newly executed hashes.
