# ARD-0103: Tree option activation at the public project seam

Upstream OrcaSlicer v2.4.2 `SupportParameters.hpp:190–204` normalizes
Tree-only styles under Normal to Grid. `TreeSupport.cpp::generate:1724–1731`
dispatches Organic separately. `drop_nodes:2657` and
`TreeSupportData::TreeSupportData:3653` consume the generic branch angle;
`TreeSupportCommon.hpp:83–92` consumes the Organic angle/distance/diameter
and preferred angle/top rate. Raw setting readback alone proves neither path.

The Rust destination for this slice is the CLI parity harness
`option_coverage/activation.rs` and its support-specific submodule, not a new
slicing pipeline. Core destinations remain the existing
`support_material/tree/{non_organic,organic}` source rewrite. Replace the
blanket Tree-auto enabler with source-owner-specific style co-injections.
Generic branch angle/diameter/distance and classic brim controls use Slim;
Organic-only controls explicitly use Organic, except the schema tip domain
which uses raw Grid normalized to Organic as explained below. Shared controls
use Organic.
Manual brim width additionally disables automatic brim
(`TreeSupport.cpp:2023,2136`); the auto-brim bool keeps its own value.
Tree-only support_style enum values select Tree-auto; Grid/Snug/default
select Normal-auto, retaining upstream default normalization coverage.

Activation siblings: branch_distance uses max_bridge_length=2 so its legal
minimum1 is not masked by sample_step=max(distance,max_bridge_length/2)
(`TreeSupport.cpp:3422`). Preferred angle uses legal Organic angle60;
raw preferred maximum85 remains source-clamped, not silently replaced.
Tip validation and normalization are distinct contracts. `Print.cpp:1580–1610`
checks explicit Organic/default raw styles: tip must be at least the support
flow width and no greater than branch diameter (schema maximum10). Explicit
Organic tip100 is rejected, not a successful clamp. Raw Grid is accepted by
this source guard, then normalized to Organic by SupportParameters; Common.hpp:92
clamps its schema tip maximum100 to branch diameter10. Preserve the full
min/max/seeded schema domain on that normalized path, without calling100 an
effective diameter or crediting the rejected explicit-Organic configuration.
Use support_line_width0.1 with layer/first-layer height0.08 for legal minimum
tip0.1 (`Print.cpp:1543–1555,1666`). Add explicit Organic finite legal-boundary
controls0.1/10 and separate native-only rejection100 evidence, not Ares error
parity or whole-range credit. Brim controls explicitly use raft_layers=0.

Included: public 3MF option-domain exports, requested value and dependency
readback, positive native support extrusion, option-sensitive native support
movement witnesses, and unchanged strict complete G-code comparisons.
Ranges retain generated min/max/seeded interiors; styles retain all values.
Use matched Ares/reference execution domains per ARD-0102; preserve earlier RED.
No production worker policy, comparator or existing fixture change.

Closure follows the bounded-checkpoint scope: preserve genuine tip
minimum fill-connection RED assertions and evidence, without representing
the activation correction as complete Tree option output parity. Production
connector repair is not implemented in this checkpoint. Full quality gates
remain mandatory; opt-in reference failures are reported separately.

Deferred: unrelated printer/Option failures, all-platform runtime and
cross-worker equivalence.
