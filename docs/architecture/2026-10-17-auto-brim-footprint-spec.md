# Auto brim footprint SPEC

## Premise result

The active fixture has two separated objects,
XY scale 0.1 and Z scale 5, `brim_type=auto_brim`, `brim_width=10`,
`skirt_type=perobject`, `combine_brims=0`, one filament/instance, and no
support, raft, or tower. To isolate Auto from the per-object admission guard,
the same bytes were changed only to `skirt_type=combined`.

On the combined input Native official and seeded modes both return rc0. Both
emit two Brim blocks and two Skirt blocks. Current Ares on the already-admitted
combined path returns rc0 but emits zero Brim blocks (two Skirt blocks).
Thus the owner is Auto brim width/geometry, not the per-object guard. The
per-object Auto admission remains a later slice after this geometry slice.

## Upstream and Rust boundaries

`Print.cpp::_make_skirt:2780-2977` and footprint grouping `2847-2971` consume
the computed brim footprint. Upstream Auto width is calculated in
`Brim.cpp:260-305` (`configBrimWidthByVolumeGroups`, including the
`height_to_area * maxSpeed` versus thermal-length formula at `298-300`), with
`getadhesionCoeff`, `Model::findMaxSpeed`, and `Model::getThermalLength`
(`Model.cpp:3258-3280`) supplying the inputs. The resulting group width is
consumed by `Brim.cpp:448-530`: `has_brim_auto` resolves each first-layer
object-volume group, rounds `brimWidthRaw` to flow spacing, applies the
`groupHeight > 10` adhesion bump and the `1.5 * bbox` clamp, and rounds the
final flow width.

Ares defaults `brim_type` to `AutoBrim` (`options/object_fields.rs:7`).
`brims.rs:100` classifies AutoBrim as an outer-brim producer, but the exact
zero-output site is `project_slice/gcode_emit/brim.rs:48-56`: its
`!matches!` admission returns `None` before width calculation because
`AutoBrim` is absent. After that guard is removed, the current width path at
`:111-130` still uses only configured `brim_width`; GREEN must replace it with
the cited volume-group calculation, feed the existing BrimPlan, and let
`skirt/occupied.rs:61-80` consume it. The per-object admission match in
`skirt/generate.rs:48-56` remains deferred until this geometry is ported.

## RED and scope

The public RED uses the combined Auto input, Native through `OrcaRunner`, and
asserts Native rc0, active Brim and Skirt blocks, then unchanged strict whole
output equality for API and CLI. It must fail now on the missing Auto Brim
geometry in both official and seeded modes. The GREEN must port
only the cited Auto width/dispatch behavior into the existing brim plan; no
per-object admission, combine-brims, instance-count, or tower/flush changes
are included. Because AutoBrim is the default, GREEN gates include the RED,
all existing per-object-skirt/brim families, and a broad existing parity subset
in both modes, plus fmt, strict clippy, and wasm32. No native artifacts are
committed.
