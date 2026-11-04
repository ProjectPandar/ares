# F1/F2 reachable group dependency: development stack

F1 public RED a31c6142c is complete. F1 GREEN development proceeds
before alternate merges.

The SPEC requires a decision for a reachable multi-group dependency
that prevents a clean F1 boundary. Its bans on owner-only configuration,
silently omitted fuzzy geometry and new Unsupported guards remain binding.

## Concrete dependency

`project_slice/perimeters/groups/compatibility.rs::is_compatible` deliberately
omits every fuzzy field, mirroring Layer.cpp perimeter compatibility.
`groups.rs::prepare_layer` collects all compatible members, chooses the
higher-density owner, and merges their surfaces. `types.rs::perimeter_surfaces`
returns that union only for the owner; `region_options` returns the record's
single region. `arachne/materialize.rs::materialize_record` currently constructs
the traversal fuzzy configuration from that owner region.

Consequently a compatible Arachne group can contain two different normalized
fuzzy configurations even though compatible_region_ids is a singleton field.
For example, equal wall settings with None versus External form one perimeter
group with two fuzzy configurations. Picking the owner's config for the union
either omits the other's fuzzy effect or fuzzifies its inactive surfaces.
Changing compatibility to split these walls would violate the upstream owner.

Upstream `FuzzySkin.cpp::group_region_by_fuzzify:415–473` iterates every actual
compatible member and compares the complete scaled fuzzy configuration.
One distinct configuration yields whole coverage at465–468; multiple configs
need safety-offset surface masks at470–473. `apply_fuzzy_skin:685–694` owns the
single-group fast path; its else branch starts effect merging at696–710 and
ordered LineSplit reconstruction at744–814. Those are exactly F2's deferred
production boundary, not a different F1 interpretation of a first region.

## Accepted decision 791a9fbe9

Bring the minimum required F2 masks/ordered-split dependency into the development
sequence before opening the public F1 entry. First commit/report separate
multi-group Native-backed RED, then port that already-source-cited boundary.
Keep F1 noise/junction and F2 mask/split production changes in distinct commits
and keep them separate on the complete public GREEN stack. Do not merge or
declare partial public F1 GREEN while valid multi-group inputs are unhandled.

This changes independent F1 GREEN sequencing:
F2 public Native RED is committed/reported first, F1 noise/junction production
is its own commit, then F2 group masks/ordered split production is its own commit.
Full parity runs once on the complete stack; F1/F2 merge together.
No guard is tightened or added,
no configuration is discarded, and no compatibility rule is changed here.
The obsolete test retirement proceeds separately.

## F1 production closure decision

PerimeterGenerator.cpp:534-540 contains the disconnected-path TODO and
closing assert, compiled out under NDEBUG. Native release evidence
(`closure-width-audit.json`)
admits Ripple gaps 0.320306 / 0.200202 mm. F1 removes only the closing
debug_assert from traverse.rs and emits the same open sampled loop;
point-count and adjacent-path joint assertions remain. No closure repair
or retagging is introduced. F2's own endpoint assignment remains the
separate source behaviour at FuzzySkin.cpp:811-812.
