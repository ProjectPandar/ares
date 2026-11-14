# Ordinary bed temperature with an unused hotter filament

These authored inputs derive from the existing synthetic
`../m140-initial-bed/unused-hotter-filament.3mf`. They are not Native exports.
`generate.py` reproduces their ZIP contents; `sha256.json` pins the bytes.
Only `Metadata/project_settings.config` changes, and its only semantic change
is `cool_plate_temp`: `["88", "63"]` for the unchanged-used case and
`["88", "43"]` for the cooler-used case. Initial targets remain `["88", "63"]`.
The object prints with logical filament 1; filament 0 is unused.

`GCode.cpp:5532–5541,6958–6967` at `40ecc66f7` selects the maximum over
first-layer-used filaments for ordinary temperatures too. The first case
must not emit a redundant layer-two M140. The second must emit the cooler
used target, rejecting unconditional command removal. The existing labelled
start/all-bed-command E2E oracle retains strict whole-output differences;
its scoped assertions do not claim whole-output parity.
