# Region-local infill combination inputs

SPEC `b7327c79a`, pinned Orca8500fcd
`PrintObject.cpp::combine_infill:4169-4237`. The RED covers the region-local
layer view/reset/actual layer-id0 skip; projection, geometry/math and Flow/
G-code consumers remain unchanged.

These six input-only ZIPs are byte-for-byte copies of independently probed
projects, not regenerated settings/geometry. All non-settings members match
the corresponding original single-/multi-region overhang input. The seventh
canonical case directly reuses
`../seam-gap-fill-group-key/absolute-equal.3mf` (no duplicate stored here).

| Case | SHA-256 |
|---|---|
| combined-sparse, target | `ca91af7af0bb5a20f631d423776d6fd0c9589a875a3bf6eb5077584dc69f38eb` |
| combined-density25, target | `be44e22474f9cb1a169175cc508c6a408f41004480ed49a5e0720a44b4df0afd` |
| reused canonical/off | `7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b` |
| combined-identical | `f9acf0b75021da64743e389446a2af099fa647edba60217b3a8386396e6761d5` |
| combined-single | `76fbbcdf669a9ba39c368c9a27724c865ced36761a315cb00274ad160ac4c6ad` |
| one-layer-distinct | `0262510728204af2e99b869e3cf63ba687068d26c6704dbbb17c7261d3d89bcb` |
| density25-without-combination | `c5a43f0c5a2e29b80e3c804b280098bb24c8c48fe6fb3426f8bc19b3e83475d7` |

The third similar suite uses the extracted shared case-runner (frozen read,
ZIP metadata, admission/repeat/activity/complete lifecycle), backed by the
unchanged complete.rs Native/API/CLI producer and strict full-output oracle.
Every test asserts PASS. Before GREEN, two targets
strictly fail and five controls pass in both canonical modes. No inverted
assertion, G-code golden, output filtering, fallback or unsupported guard.
