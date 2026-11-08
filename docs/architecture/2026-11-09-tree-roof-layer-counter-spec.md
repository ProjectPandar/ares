# Tree support roof-layer counter and first-roof fill angle

## Rewrite boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream | Behavior | Rust destination |
| --- | --- | --- |
| `Support/TreeSupport.cpp:3706,3764,3770` (`generate_contact_points`) | a contact node is created with `roof_layers = support_interface_top_layers`, or 0 without an interface | `support_material/tree/non_organic/contacts.rs:135-139` |
| `TreeSupport.cpp:3122` (merge) and `:3207,:3368` (drop) | `support_roof_layers_below` decrements when `distance_to_top >= 0` | `non_organic/merging.rs:137`, `non_organic/dropping.rs:203` |
| `TreeSupport.cpp:3624-3650` (`plan_layer_heights`) | rewrites height and `distance_to_top`; never writes `support_roof_layers_below` | `non_organic/heights.rs:61-76` |
| `TreeSupport.cpp:2332-2350` (`draw_circles`) | classifies a node by `support_roof_layers_below` and `is_sharp_tail` only | `non_organic/drawing/areas.rs:49-66` |
| `TreeSupport.cpp:1728`, `SupportParameters.hpp:308-327` | the first roof fills at `support_interface_angle(interface_id)` | `non_organic/toolpaths/interfaces.rs:93`, through `toolpath_parameters.rs:158` |

This is a port of the upstream counter and angle, not a new support pipeline.
The Rust seed adjustment, height override and millimetre cap below are a
temporary compatibility shell around `support_roof_layers_below`, and they are
removed rather than kept beside the ported counter.

## Measured owner

Fixture: `tree-support/slim/contained-travel`, Tree Slim on the Ender-3 profile
with independent support layer heights and three top interface layers. Same-layer
captures were taken on both sides at `GCode::extrude_support`
(`GCode.cpp:8973-9021`) and at the const overload of
`chain_and_reorder_extrusion_entities` (`ShortestPath.cpp:1513-1528`). Native was
read through the debugger on the existing seeded binary, whose G-code body is
identical to the official reference.

- All but two support layers match on incoming entity order, nested no-sort groups,
  chain start and post-chain order. Native `no_sort` is false for every top-level
  call (`TreeSupport.cpp:1686`); Rust has no such layer flag and always chains,
  which agrees.
- On the two remaining layers the incoming entities already differ, so the chain
  is not the owner: it receives different input.
- The chain port produces the Native order on every layer once its input matches.

Both differences are upstream of `extrude_support`:

1. **Roof count.** Rust classifies one extra top layer as roof, which moves the
   first-roof layer one layer down and replaces a base layer on the fixture.
2. **First-roof angle.** The first-roof fill runs perpendicular to Native, because
   Rust uses `support_angle` where upstream uses the interface angle.

Node positions and branch geometry are not involved. With only these inputs
changed, the whole G-code stream matches Native, including `M73`, apart from the
generator line and two configuration rows absent from that CLI invocation, neither
of which affects motion.

## Rust divergence

| Site | Rust today | Upstream |
| --- | --- | --- |
| `contacts.rs:136` | seeds `(roof_layers - 1).max(0)` | seeds `roof_layers` |
| `merging.rs:137`, `dropping.rs:203` | decrements on `distance_to_top > 0` | decrements on `distance_to_top >= 0` |
| `heights.rs:61-76` | recomputes the counter from millimetres | no such write |
| `areas.rs:49-50,55,61` | adds `within_roof_cap` | no such condition |
| `interfaces.rs:93` | first-roof `base_angle()` | `support_interface_angle(interface_id)` |

## Change

1. Seed `roof_layers` at the contact node.
2. Decrement on `distance_to_top >= 0` in `next_seed` and in the merge node.
3. Delete the counter override in `plan_layer_heights`, and the
   `top_interface_layers` input field that it alone reads.
4. Delete `within_roof_cap`. `drawing/areas.rs:96` keeps its own
   `top_interface_layers` read, as `TreeSupport.cpp:2317` does.
5. Set the first-roof angle through the existing `support_interface_angle` helper
   with `group.interface_id`.

The counter changes form one correction and are not applied separately: measured
with the `>= 0` decrement and the angle but without the seed and override changes,
a passing roof witness regresses.

## Included and deferred

Included: the five items above, for non-organic Tree Slim, Strong and Hybrid.

Deferred, each with no witness in this slice and needing its own failing case:

- `TreeSupport.cpp:2343,2359-2366,2571-2576,2585`: `roof_base_areas` and
  `first_base_roof_area` drive `interface_as_base`, where `drawing/groups.rs:109`
  derives it from `roof_id`.
- `SupportParameters.hpp:316-318`: the interlaced roof angle is the interface angle
  plus or minus a quarter turn, where `interfaces.rs:122-125` uses the base angle
  plus `(interface_id & 1)` quarter turns.
- `ShortestPath.cpp:1513-1528`, `GCode.cpp:8952,9021`: the const overload reverses
  clones, so reversals do not reach the shared entities. `shared_reversals.rs` and
  `entity_chain.rs:36-38` persist them across instances and cite the earlier mutable
  behavior. Only multi-instance prints can observe this.

Organic tree support, `SupportMaterial`, and branch geometry are out of scope.

## Failing cases and acceptance

No new fixture, stored coordinate or stored count is added. Measured with the change as a temporary experiment on
the `tree | support` test selection, official and seeded modes giving the same
result, against the unchanged tree:

| Group | Result |
| --- | --- |
| Slim, Strong and Hybrid auto overhang complete output | fail to pass |
| Manual blocker, split, split-side and style complete output | fail to pass |
| Support islands of the smooth Type 2 tower | fail to pass |
| Tree retraction witnesses, including the support loop and travel vectors | fail to pass |
| Pass to fail | none |

The retraction witnesses live on the premise-scoped test commit, so this change
stacks on that commit or follows it onto the main line.

Remaining failures in the selection were failing before the change and are not
claimed, among them the Tree lightning, base-connect, default short-edge and
base-extension cases. Attribution within the three counter deletions (seed, override,
cap) is by source comparison only: they were measured together.

Implementation order: run the listed cases on the unchanged tree and record the
failures, apply the change, rerun both modes, then run the full suite.
