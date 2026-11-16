# Config-definition contextual message decoding

## Source and destination

Pinned OrcaSlicer: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
`libslic3r/I18N.hpp:29-32` defines `L_CONTEXT(s, context)` to return `s`;
context is a translation discriminator, not part of the English message.
`PrintConfig.cpp` uses this for seconds units and input-shaping axis labels.

Port that expression boundary into `scripts/gen-configdef/cpp.mjs` using its
existing balanced-call and argument splitting functions. Ordinary adjacent
C++ string literals must still concatenate. The generator is a source-reader,
not a second configuration policy. No runtime slicer or typed option change
belongs here.

## Failure inventory and checks

The approved seams are the exported literal decoder and the real generator
`--dump` CLI followed by the existing Rust metadata renderer.

- Context text must not enter msgids: `ssecond`, `XAxis`, and `YAxis` are wrong.
- Adjacent msgid literals must concatenate without consuming the context.
- Escapes, commas, parentheses inside literals, and call whitespace must not
  confuse argument boundaries.
- Ordinary literal sequences and ordinary `L(...)` calls remain unchanged.
- Full pinned generation must change only the 17 contextual fields. Twelve
  previously drifting blocks must become byte-identical to committed metadata.
- Four records already contain incorrect generated units in committed metadata:
  `machine_max_force_Y` (`PrintConfig.cpp:5477-5485`), `machine_bed_mass_Y`
  (`5487-5495`), `machine_max_printed_mass` (`5497-5505`), and
  `machine_hotend_change_time` (`8819-8825`). Promote exactly these four blocks
  from the corrected generator output; their `L_CONTEXT` first arguments are
  the owning source values.
- `fan_cooling_layer_time` also has an unrelated tooltip gap: correct its
  generated unit without promoting that block or changing its tooltip.

## Bounded output

The 12 context-only records are already source-equivalent; regenerating them
must be a byte-identical operation. Correct exactly the four additional blocks
listed above. The remaining full-generation drift must fall from 235 to 223
keys. Without those four corrections, the generic decoder exposes four latent
committed errors and yields 227 drifting keys; do not hide that intermediate
result. The mixed cooling-fan record remains in the final drift set. No
unrelated metadata, enum state, default, key ordering, parser rule, runtime
behavior, or dependency belongs in this slice.

## Public API oracle

`OrcaSlicer.cpp:8318-8327` comments out `print_config_def.print_cli_help`;
`--export-settings` exports configuration values, not metadata. There is no
Native runtime metadata oracle for these observations, and no patched Native
or debugger probe is introduced.

The public API E2E runs the real generator on the pinned source and compares
its relevant field for all 16 records with `ares_core::print_config_def_json()`.
Expected field values come from that generation, never hardcoded units. At the
unchanged core baseline the twelve existing records pass and the four incorrect
units fail. After promoting only those four blocks, all sixteen must pass.
Retain source identity, generated JSON, public JSON, and field comparisons as
repeatable artifacts. This is source-oracle metadata parity, not a Native
runtime execution or WASM/browser claim.

The generator CLI check still reports inherited metadata drift; a globally
clean `--check` is not an exit criterion. Retain reproducible JSON, generated
Rust, per-block hashes, and the complete drift inventory outside the repository.

Run focused generator checks with:

```sh
node --test scripts/gen-configdef/cpp.test.mjs
ARES_CONFIGDEF_TEST_ARTIFACTS=/absolute/fresh/directory \
  node --test scripts/gen-configdef/context-metadata.test.mjs
```
