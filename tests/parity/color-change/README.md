# ColorChange processor oracle fixtures

These checked-in G-code inputs probe the
[`web-colorchange` spec](../../../docs/architecture/web-colorchange.md).
They are native processor inputs, not an alternative slicing/emission path.
Future slice/browser fixtures must use existing user G-code template overrides.

These producer-less inputs use the **BBL spaced tag family**:
`GCodeProcessor.cpp:116` initializes `s_IsBBLPrinter = true` independently of
G-code flavor; `GCodeProcessor.hpp:375` selects the tag family with that flag.
Only a recognized producer with a nonempty `printer_model` in its embedded
config changes it (`GCodeProcessor.cpp:2620-2633`). The first diagnostic run
used compatible tags and therefore recorded no events; that invalid setup is
preserved separately in artifacts. Correcting tag spelling does not change
any source-derived expected cardinality below. The processed-slice RED uses
an actual Ender3 project and its compatible user-template tags.

## Failure inventory (before Rust implementation)

Upstream `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

- `GCodeProcessor.cpp:3237-3301`: a standalone `COLOR_CHANGE` tag creates the
  active-filament event; M600 alone does not. Events with no later extrusion
  remain pending for Z correction (`GCodeProcessor.hpp:677-711`).
- `:1534-1555, 1641-1648, 5936-5949`: only model extrusion adds color volume;
  active ColorChange and EOF flush nonzero values. Travel/support add none.
- `:2702-2707, 5917-5934`: a tag flushes nonzero custom time; EOF supplies
  the trailing ColorChange time record. Immediate consecutive tags have no
  intervening time and therefore need not produce matching time records.
- `GCodeViewer.cpp:4221,4230`: each ColorChange time record reads one volume
  without a bounds check, whether or not a corresponding custom item remains.
  Equal vector lengths alone do not prove event-to-time correspondence.

Source-derived cardinalities to check with the native oracle:

| Fixture | Custom items | Nonzero model volumes | ColorChange time records |
| --- | ---: | ---: | ---: |
| normal | 2 | 3 | 3 |
| no-change | 0 | 1 | 0 |
| bare-m600 | 0 | 1 | 0 |
| zero-before | 1 | 1 | 2 |
| zero-between | 2 | 2 | 3 |
| zero-after | 2 | 1 | 2 |
| consecutive | 2 | 2 | 2 |
| zero-only | 1 | 0 | 2 |
| support-only | 1 | 0 | 2 |

Each model interval consists of two XY extrusion moves with E=1 in relative
mode. Nonempty travel intervals use two moves so they are not mistaken for an
empty time interval. `normal` prints three colors; `consecutive` retains two
custom items but prints no material in the intermediate red color. No assertion
may be updated merely to match an Ares or browser observation.

The probe audits only the source's vector reads. It does **not** execute the
commented-out GUI table, claim a native GUI crash, silently fill missing
volumes, or zip/truncate records. An out-of-bounds index is evidence that the
source display join needs an explicit decision before its Rust port.

## Native outcome and display boundary

The historical Native probe matched all nine cardinalities in the source-derived
table above. Its frozen JSON summary has been removed; the CLI runtime E2E
`runner::adjacent_color::committed_bbl_cardinality_fixtures_match_native_inside_start_templates`
compares the original nine handwritten templates embedded in real slices against
explicit raw/stable processor diagnostics. Native output and receipts stay in
OrcaRunner runtime/cache directories. Five fixtures require out-of-bounds
volume indices in the upstream display join. The decision is:
preserve raw statistics and ColorPrint segments; Rust reports partial usage
unavailable when a required volume index is absent, and the frontend displays
that status instead of guessing or partially returning usage rows. A no-change
print needs no partial-row volume reads and is not unavailable.

The static `apply_config(PrintConfig)` probe overload returns empty
`extruder_colors` even with `filament_colour` configured; these raw fixtures
prove cardinality, not the processed project's palette configuration. Palette
parity will use the native generated G-code/config and processed-slice seam.

## Reproduce

Build the diagnostic binary with `tools/chain-probe/procgcode.nix`, following
its pinned-source and nixpkgs instructions. The patch only adds diagnostic
configuration/output to CLI main; it does not change the upstream processor.
Then, from the repository root:

```sh
node tools/chain-probe/color-change.mjs \
  --oracle /path/to/orca-slicer-procgcode/bin/orca-slicer \
  --output target/color-change-oracle
```

`--colorchange-tools 1` explicitly configures Marlin Legacy, one 1.75 mm PLA
filament at 1.24 g/cm³, color `#F2754E`, a 0.4 mm nozzle, and SEMM off. Machine
caps retain the existing probe recipe. Raw `.json`, move `.csv`, logs and
`report.json` record commands, fixture hashes, native executable/patch hashes,
counts and any would-be out-of-bounds GUI volume indices. The probe accepts a
different output directory; input files always come from this checked-in
fixture directory, never an external temporary directory.
