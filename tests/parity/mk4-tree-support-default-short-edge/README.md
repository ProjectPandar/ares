# MK4 organic default support seam anchor fixtures

Stock Prusa MK4 0.4-mm overhang project with the established support family,
`tree(auto)`, default (Organic) style, wall count 0, support width 0.42 mm,
and interface top layers 4. Frozen from the single-key support matrix and
its strict one-key control; geometry and all other configuration are equal.

- `default.3mf` — `support_base_pattern=default`; SHA-256
  `13b069f79fb3aebdddf9fd077298e80fed821247d1ee9bd855e35178bb058964`.
- `rectilinear-control.3mf` — only `support_base_pattern=rectilinear`; SHA-256
  `339351a56db7c4df91e5f13aa2d358442ae50d6085e1ffa2a485e91e86ce0e00`.

At OrcaSlicer `345efec64`, `SupportCommon.cpp:1955-1958` routes Organic default
support to `tree_supports_generate_paths` (`:642-843`), which cuts a seam anchor
from a second loop made by its shrink helper (`:651-707`). The rectilinear
control does not take that route.

These fixtures do not cover the shrink helper's short-edge threshold. They were
recorded when it decided the anchor on one layer, but at this revision no edge
falls under it, and the output is the same with the filter disabled.

Both cases use `native_stream_case::check` for Native double-replay and
public API/CLI complete ordered byte comparison, normalizing only the
generator line. No Native output or probe is committed.
