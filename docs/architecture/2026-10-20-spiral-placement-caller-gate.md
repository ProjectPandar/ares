# ARD: configured spiral mode bypasses perimeter SeamPlacer placement

Status: SPEC approved by COORD; formal RED authorized, GREEN requires
separate review approval. Base d592d41459764d816c0199385a22b442006cba76.
Owner W5, emitter/placement, not W6's GCodeProcessor seam cache/scarf flag.

## Upstream boundary and Rust correspondence

OrcaSlicer2.4.2 `src/libslic3r/GCode.cpp::GCode::extrude_loop:5769-5775`:

```cpp
if (!m_config.spiral_mode && description == "perimeter") {
    assert(m_layer != nullptr);
    m_seam_placer.place_seam(m_layer, loop, last_pos, seam_overhang);
} else
    loop.split_at(last_pos, false);
```

Both conjuncts matter. Rust represents `description == "perimeter"`
structurally: `seam_placement::placement::place_layer` selects only
`IslandPrintEntity::Perimeter` collections, excluding Fill,
FillCollection and Thin; `place_collection` selects only
`ExtrusionEntity::Loop`, excluding Path and MultiPath. The emitter's
`motion/perimeter.rs::emit_perimeter` has the same perimeter-only entry
and directs open entities to their own emitters. Skirt/brim and fill
are not an invitation to run this perimeter placement pass. Preserve
this existing dispatch restriction, rather than applying placement to
all loops or treating an extrusion role as the description string.

The missing conjunct is the configured `!spiral_mode` gate, independent
of whether the current layer is actively transformed by SpiralVase.
Rust destination: `seam_placement/runtime.rs::placement_modes` and only
necessary existing placement dispatch. `seam_placement.rs::apply` uses
these modes before preparing/alignment/splitting. Existing
`motion/perimeter.rs` spiral nearest projection corresponds to the
upstream `split_at(last_pos,false)` else branch; preserve it.

## Replace the temporary CW-axis rectangle shell

Current `placement_modes` only bypasses aligned placement for Clockwise
axis rectangles, using `(spiral_mode || !conditional_scarf_condition)`.
The spiral arm is a temporary compatibility shell, not an upstream
shape rule. Replace it with the general configured-spiral caller gate
before ordinary mode selection. Remove the spiral OR-arm, not add a
cylinder/point-count/direction exception or retain the old spiral arm
as a fallback. CW rectangles must pass through the SAME uniform gate.
Preserve independent nonspiral shortcut behavior; this witness does not
justify ordinary aligned or conditional-scarf redesign.

## Native-first normally reachable witness

GUI `PrintConfig.cpp:5829+` defines coBool `spiral_mode`, label Spiral
vase. Retained input is a closed octagonal cylinder radius6mm,
height1.2mm,18 vertices/32 triangles, six layers. Geometry is a normal
importable mesh, not malformed configuration. Inputs were actually
exported by official CLI, then sliced from those exports.

Common Classic/one wall/0% sparse/no top or bottom shells,0.2mm layers,
aligned/staggerOFF/scarfNONE/wipeOFF/no brim/skirt/arcOFF. Both layer-change
retraction settings are0, including the filament override, so Native's
normalization side effects do not contaminate the sole-key pair.

| Actual exported input | SHA256 |
|---|---|
| control spiral0 | 89e9b81620b4b3d210f3ae894e6041f03d4638e40ce240d134614e9e866095e0 |
| positive spiral1 | aa361aa6fe7c0a5bb03a5eb8e891e01ffd04cba687d427d35db1f25fea24bed4 |

Only ZIP payload project_settings.config differs, with only spiral_mode
0→1. Official original/repeat and seeded Native complete streams match
apart from the canonical generator identity/date line. Control matches
Ares; positive has828 aligned differences in both modes, first at71:

```
Native: G1 X110 Y104.227 E.13576
Ares:   G1 X109.999 Y104.228 E.13572
```

Ares additionally emits `G1 X110 Y104.227 E.00004`. No M73, E, timing,
configuration, movement or trailing-line mask is allowed.

## Read-only source proof

External evidence `${COORD_ROOT}/evidence/w5-spiral-placement-gate/`
contains actual inputs, launches, full streams, raw snapshots and SHA256
manifest. Original work root `/tmp/w5-discovery-d592-spiral3/`.

- All six complete ordered raw PG wall records EXACT, including scaled
  vertices/cyclic origin, roles/inset/loop role and flow metadata.
- SpiralVase enabled flags both `[false,true,true,true,true,true]`;
  input already differs on first layer while both filters are disabled.
  Filter arithmetic/activation are not the first owning boundary.
- Native six GCode::extrude_loop visits, spiral configured true, zero
  SeamPlacer::place_seam calls; Rust six eager placement::place_loop calls.
- Nonspiral Native counter-control has six exact place_seam entries;
  broad-symbol lambda destructor hits are separately recorded, excluded.
- CW-axis rectangle spiral complete PASS, zero eager Rust calls.
  CCW-axis rectangle PASS but six calls: matching geometry alone does not
  prove correct source policy. CW regression must remain in formal RED.
- Valid debugger full outputs equal respective non-debug outputs. No
  inferior calls/register writes/memory writes; failed optimized-local
  inspection attempts are not proof.

Restoring the gate alone has NOT been tested or claimed sufficient.

## Included / deferred

Included after separate GREEN approval: uniform configured-spiral caller
bypass, removing its shape-specific shell, preserving perimeter-only
structural dispatch and existing orientation/projected split behavior.
No new public API, dependency, I/O, fallback or Rust file over400LOC.

Deferred: W6 processor flags/cache; W2 brim/EFC/material; W3 combine-solid
and Arachne heights; W4 ironing; W1 Voronoi. Also deferred: SpiralVase
smoothing/taper/short-segment arithmetic, travel/wipe/retraction lifecycle,
ordinary seam ranking, random/hash/stagger math, conditional scarf and
nonspiral rectangle cleanup. This is a cited libslic3r caller-gate rewrite,
not an independently designed Ares pipeline.

## Formal RED then separately authorized GREEN

Freeze control/positive and CW-axis rectangle actual exports as INPUTS
only, with hashes/provenance. Reuse main's unmodified
`runner/native_api_cli.rs::NativeCase`; invoke Native twice per case,
compare complete ordered Native repeats, then full public validated API
and bare CLI output. Keep all launch/output/comparison artifacts.
Native availability only, no borrowed reference gate/helper copy.

Dual official/seeded RED must show control/CW PASS and positive FAIL at
the real motion premise, no ignored tests or expected-failure conversion.
Runner module remains below400LOC. Send REVIEW-REQUEST and WAIT for
approval before production changes. Future GREEN must make spiral eager
placement count zero universally, preserve ordinary placement, and keep
CW passing via uniform gate. Residual mismatch blocks scope expansion.
FULL only in a separately authorized window; no discovery probe is
formal public-API or current-main integration evidence.
