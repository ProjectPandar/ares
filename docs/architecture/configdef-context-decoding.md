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
- Full pinned generation must change only the 13 contextual fields. Twelve
  previously drifting blocks must become byte-identical to committed metadata.
- `fan_cooling_layer_time` also has an unrelated tooltip gap: correct its
  generated unit without promoting that block or changing its tooltip.

## Bounded output

Do not rewrite committed metadata. Its 12 context-only records are already
source-equivalent; regenerating them must be a byte-identical operation.
The remaining full-generation drift must fall from 235 to 223 keys, with all
other generated fields unchanged. The mixed cooling-fan record remains in
that drift set. No unrelated metadata, enum state, default, key ordering,
parser rule, runtime behavior, or dependency belongs in this slice.

The generator CLI check still reports inherited metadata drift; a globally
clean `--check` is not an exit criterion. Retain reproducible JSON, generated
Rust, per-block hashes, and the complete drift inventory outside the repository.

Run focused generator checks with:

```sh
node --test scripts/gen-configdef/cpp.test.mjs
ARES_CONFIGDEF_TEST_ARTIFACTS=/absolute/fresh/directory \
  node --test scripts/gen-configdef/context-metadata.test.mjs
```
