# Type2 in-place toolchange gap entry

## Native boundary

This slice translates OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `GCode.cpp::WipeTowerIntegration::append_tcr2:1690–1713` computes the travel gates; `:1762–1789` handles entry after an in-place toolchange.
- `GCode/GCodeProcessor.cpp::get_last_position_from_gcode:4310–4399` recovers the terminal position from the emitted toolchange string.
- `GCode.cpp::GCode::gcode_to_point:11496–11504` converts the recovered position into the object's integer plane.
- `GCode.cpp::WipeTowerIntegration::travel_to_tower_gap:1029–1069` supplies the intermediate entry waypoints.

The second call site deferred by `wipe-tower2-gap-travel.md` is the boundary here. The existing pre-toolchange approach remains unchanged.

## Entry-state correspondence

No Rust behavior change is permitted until the existing tools fixture demonstrates these Native/Rust operands equal at the same toolchange, immediately after tool selection and before gap-entry travel:

| Operand | Correspondence |
| --- | --- |
| Result identity | Layer, initial tool, incoming tool, result index and priming flag |
| Travel predicates | `force_travel`, `needs_toolchange`, `will_go_down`, `is_ramming`, `tool_change_on_wipe_tower`, `should_travel_to_tower`, layer-change lift request and `travel_to_tower_now` |
| Branch predicate | `!travel_to_tower_now && !tcr.priming && use_gap_wall` |
| Position seed | Writer XYZ cast componentwise to `f32` after the custom toolchange |
| Position scan | Exact toolchange-string bytes, parser boolean and resulting XYZ bits |
| Fallback | Defined-last-position flag and integer last position when parsing returns false |
| Selected route start | Selected parser/fallback arm and final integer `route_start` |
| Route destination | `tcr.start_pos`, tower position/rotation/rib offset, plate origin, object origin, active nozzle offset and integer `start_wipe_pos` |
| Route geometry | Tower depth/width, first-layer box, body and avoid boxes, shared printable polygons and gap-wall settings |

A completed table must contain actual operands, not values inferred from the desired output. Float equality uses exact bits and integer points compare exactly. Different tools, entry positions or earlier planning operands are separate causes to localize before this call site can change. The branch predicate must be true on the target; an inactive predicate cannot establish the missing behavior.

The retained whole-output failure is earlier M73 text. It does not establish a timing-producer cause or any of the state equalities above.

## Position recovery

There is no complete Rust equivalent of `get_last_position_from_gcode` in the inspected tower code. `wipe_tower/type1/travel.rs::path` already contains its XY-only scan, seeded with zeros, but neither its XYZ seed/result nor the success boolean required here.

Extract that existing scan into a shared private `wipe_tower/last_position.rs` helper, extend the same Native axis logic to Z, and return the Native success boolean. The Type1 caller keeps its zero seed, ignores the boolean and consumes only XY; its routing and point conversion must remain byte-neutral. This is reuse of the existing parser, not a second decoder or a generic G-code interpreter.

Preserve the pinned scan semantics:

- Consume newline-terminated complete lines only; retain the existing leading-space and trailing-semicolon/space trimming sequence.
- Inspect only lines longer than five bytes beginning with `G0 `, `G1 `, `G2 ` or `G3 `.
- Find the first ` X`, ` Y` and ` Z` word independently and update only successfully parsed axes as `f32`.
- Preserve the existing eight-byte numeric bound and word terminators, following the pinned Native pointer-width bound on its 64-bit build. Do not reinterpret relative modes, reset commands, compact words or unterminated trailing text.
- Return true for any successfully parsed axis, including Z-only; keep unspecified axes from the caller's seed. A Z-only success must not be mistaken for parser failure.

The new caller seeds from the post-toolchange writer XYZ cast to `f32`. On success, promote XY to double, add the plate origin in double and use `gcode_to_point`: subtract object origin, add the selected nozzle offset, then scale with truncation. This is not the rounded tower-point conversion. On parser failure, use the defined integer last position; if that position is undefined, emit no entry travel.

## Rust boundary and operation order

`wipe_tower/emit.rs::append` is the owner of this missing `append_tcr2` branch. It currently passes from `tool_transition::tower` directly to optional Z restoration. This is an incomplete translation of the Native function, not an independent pipeline.

1. Retain `travel_to_tower_now` from the existing pre-toolchange predicate, including the layer-change request.
2. After `tool_transition::tower`, and before `motion::tower::restore_z`, apply the exact branch predicate and recover `route_start` as above.
3. With a defined start, set integer last position and select the external perimeter map once.
4. Reuse `wipe_tower/tower_gap.rs::waypoints` with that start, the existing transformed destination, transform and layer geometry. Do not duplicate or modify its router, body test, clipping, bed intersection or point ordering.
5. Emit every returned waypoint, then `start_wipe_pos`, through the existing `motion/path/start_travel.rs::tower_travel` operation. Append these bytes, with Native's line-ending rule, to the toolchange substitution rather than directly to the outer output.
6. Set integer last position to `start_wipe_pos` before the existing Z-restoration/deretraction generation and template substitution continue.

Add only the narrow `motion/tower.rs` bridge needed to access the existing travel primitive and last-position state. The current `motion::tower::travel_to` cannot be reused unchanged: it also performs explicit unlift/unretract after travel, which this Native branch does not add. Retraction caused by ordinary `travel_to` remains owned by that primitive; do not add a separate retract operation or change lift policy.

## Scope and dependency

Included: the second call site, shared terminal-position recovery, its selected-position lifetime, the existing gap router's invocation and the resolved current-tool retraction-speed parameter described below. Reuse existing typed configuration and writer state; introduce no configuration defaults or shadow readers.

Deferred: tower plan/depth, first-line credit, purge-row lattice, entry-scrub geometry, ramming, boundary wipe start, grouping, final purge, lift policy, geometry precision, processor timing and unrelated template parsing.

The branch dispatch and position recovery do not own after-wipe reservation. Establish entry-state correspondence on the exact source/configuration being translated; do not transfer comparisons across different parameter producers.

## Test premise

Use the existing public test `smooth_type2_tower::tools::smooth_type2_tower_tools_and_priming`, without fixture, expectation or comparator edits. Its `type2-tools/idex` member exercises two physical tools without front-edge priming. Confirm the branch's actual ramming and gap predicates rather than inferring them from the test name.

Retained outputs first differ strictly at byte 470, line 18, column 10: Native `M73 P0 R27`, Rust `M73 P0 R24`. Diagnostic command alignment first differs after T1/waits/pressure advance: Native retracts at line 321 and routes through the gap, while Rust begins the generated positioning move at line 320. Neither diagnostic command filtering nor a matched local travel section establishes complete-output parity.

Keep all nine existing tools/priming members and both provider modes. Compare complete ordered Native/API output, including M73, metadata and configuration, with only the existing generator normalization. Record API/CLI raw equality separately; retain pre-existing footer differences without changing footer production or either comparator. Preserve the Type1 route caller and the existing gap-travel and tower-lift controls. Any first difference left afterward remains a failure with its original and new raw context reported separately. No section-only or timing waiver substitutes for this test.

## Resolved current-tool scrub retraction speed

`PrintApply.cpp::print_config_diffs:282–317` resolves nullable filament retract overrides; `:1666–1670` applies them to `m_config`. `Print.cpp:5509–5517` passes that configuration to `WipeTower2::set_extruder`. `GCode/WipeTower2.cpp:1222–1223` stores `config.retraction_speed.get_at(idx)` in the filament parameter, narrowing to float. Its entry scrub at `:1940–1949` reads `m_filpar[m_current_tool]`, multiplies the stored speed by `60.f`, then uses that feedrate for both retract and restore.

Rust `wipe_tower/wipe.rs::scrub_entry` already selects `self.filaments[self.current_tool]` and multiplies its stored speed by `60.0_f32`. `tool_change.rs::change` selects the incoming tool before this operation. The mismatch is earlier: `planning.rs::prepare` constructs `Filament` from `views.full`, so `filament.rs::from_config` reads machine `retraction_speed` without its filament override. This is an incomplete `set_extruder` translation, not a deretraction-speed choice or a new scrub algorithm.

Supply only the selected slot's already-resolved `views.runtime.project.gcode.retraction_speed` to the Type2 filament constructor, using the existing vector-access semantics and a direct `f64`→`f32` cast. Keep the other full-view fields unchanged. Reuse `project_config_views/retract.rs` for nullable overrides and filament-map selection; do not duplicate those rules in tower code, use motion-writer integer rounding, add defaults, read `deretraction_speed` or hardcode a target feedrate. Existing consumers retain this single parameter's lifetime and the scrub retains its sign, operation order and float multiplication.

Use the existing nine-member tools test as the failing public boundary before changing this producer. Preserve the eight passing members' raw streams and all Type1 streams in both modes, and rerun the existing gap/lift controls. Whole-output Native comparison remains strict; any remaining mismatch, including M73, stays a failure.
