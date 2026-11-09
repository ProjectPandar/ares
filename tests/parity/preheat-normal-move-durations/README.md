# Normal per-move duration from X2D geometry

The public test is
`runner::processor_time::preheat_normal_move_durations::preheat_normal_move_durations_from_x2d_geometry`.
It compares complete ordered public processor move identities and Normal duration
bits against `GCodeProcessor::TimeMachine::calculate_time`
(`GCode/GCodeProcessor.cpp:630-646`) in OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c`. It also compares the real Native
`libvgcode::convert` vertices (`LibVGCodeWrapper.cpp:286-320`), including phantom
starts with zero time. No Native expected times, G-code, or converted vertices
are committed.

## Input

`x2d-two-prisms.3mf` is the generated geometry/configuration **input**. The two
closed prisms have 8 vertices and 12 triangles each:

- 12 × 12 × 0.6 mm, minimum XY `(30, 40)`;
- 80 × 40 × 0.6 mm, minimum XY `(80, 40)`.

The input uses the stock Bambu Lab X2D 0.4 nozzle machine, 0.20 mm Standard
process, and two instances of its compatible Bambu PLA Basic preset. The
project and plate both select Manual physical-head mapping `[1, 2]`. The
object filament assignments are respectively 1 and 2, with by-layer printing
and 0.2 mm first/subsequent layers. No existing fixture supplies any geometry.
`manifest.json` records consumed upstream profile hashes, the input hash and
all ZIP payload hashes.

Generate in a new external directory with the matched Native environment:

```sh
python3 tests/parity/preheat-normal-move-durations/prepare.py \
  --source /path/to/clean/pinned/OrcaSlicer \
  --binary "$ARES_ORCA_BIN" --output /new/external/directory
```

The script authors its own standard 3MF meshes and plate memberships, flattens
the selected upstream profile inputs, and asks Native to export the input
project. It does not calculate timing expectations. Its export log and original
export archive remain in that external directory.

## Live Native comparison

Set `ARES_PREHEAT_MOVE_DURATIONS_BIN` to the pinned read-only probe and run the
exact test with `cargo nextest run -p ares-cli --test orca_parity -E
 'test(preheat_normal_move_durations_from_x2d_geometry)'`. Use
`ARES_REQUIRE_ORCA=1` to require Native rather than skipping absent suppliers.

The probe source, extra patch, recipe, clean upstream tree/tar hashes and build
logs are retained outside the repository. They are not
older-source probe outputs relabeled as this upstream revision. The Native
import pass resolves only the actual public nozzle-grouping context from the
same G-code config. A separate public streaming processor loads that config,
replays the original bytes, calls `finalize(false)`, and supplies both the
observed moves and the real conversion. It does not run GL or the GUI viewer.

Each test creates a retained artifact directory with the input, complete Native
G-code, commands/exits, probe stdout/stderr, Native move/vertex JSON, applied
config JSON, public Rust move/vertex JSON and complete processed G-code. It
checks every identity before every duration bit; preceding parser/planner
identity mismatches must be localized rather than filtered out. Rust Stealth
remains zero and is not populated from Normal.
