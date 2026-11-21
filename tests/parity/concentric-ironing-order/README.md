# concentric-ironing-order — frozen input

Single-object 3mf used only as input by
`crates/ares-cli/tests/orca_parity/runner/concentric_ironing_order.rs`.

- `project.3mf` sha256
  `7143117d16ff40c408a30ecaba3abf504aa22aa64cd777b399834ff1d464dc6c`
- Derived from `tests/parity/summary/time-cost.3mf` by setting exactly these
  reachable values in `Metadata/project_settings.config`:

  | key | value |
  | --- | --- |
  | `ironing_type` | `top` |
  | `ironing_pattern` | `concentric` |
  | `ironing_angle` | `0` |
  | `ironing_spacing` | `0.15` |
  | `ironing_inset` | `0.21` |
  | `gcode_label_objects` | `0` |

`gcode_label_objects=0` is a real reachable setting: it makes upstream emit
`; start printing object, unique label id: <identify_id>` (`GCode.cpp:5356-5365`)
driven by `ModelInstance::identify_id`, which is deterministic across oracles.
It is a fixture choice, **not** an output mask.

The Top control slices `project.3mf` unchanged. The AllSolid witness derives
its input at test time by changing only `ironing_type` from `top` to `solid`
in `Metadata/project_settings.config`; it shares the same model and settings.

## Divergence pinned by this fixture

Both oracles agree with each other; Ares differs. First difference is at byte
`6016866` (G-code line `257367`):

```
expected  G1 X162.816 Y88.715 F60000
actual    G1 X162.816 Y103.715 F60000
```

Native 6,799,948 B vs Ares 6,799,928 B. The diverging layer is
`; Z_HEIGHT: 52` (not 51.2 — that layer is byte-identical on both sides).

## Source boundary

- Upstream: `Fill.cpp:1634` picks `FillConcentric`; `Fill.cpp:1714-1731` pushes
  one `no_sort` collection per `ironing_areas` expolygon;
  `ExtrusionEntityCollection.hpp:78-85,118-123` makes `no_sort`
  `can_reverse() == false`, so `chained_path_from` returns it unchanged and
  `first_point`/`last_point` are `entities.front()`/`back()`. Emitted ring order
  is the stored fill order from `FillConcentric::_fill_surface_single`.
- Ares: `project_slice/fill_entities/ironing.rs:118-167` builds the polylines via
  `crate::fill::concentric_sparse::fill_concentric_sparse` and stores them in that
  order with `no_sort: true`.

Measured on the diverging layer: `ironing_areas.len() == 1` on the Ares side, one
collection of 182 closed rings, one chain call with `n = 1` (nothing reorderable),
and identical ironing segment counts on both sides (1474) with a different maximum
ring-to-ring link distance (Native 42.910 vs Ares 34.000). Same area, same ring
set, different stored order.
