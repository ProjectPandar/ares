# Vertex zero-default public RED

Approved SPEC `cfa86d30866063ca478775414945bb9fcd377d91`; RED authorized.
Source boundary: Orca `8500fcdccaa10b5099ac20d252af3a7c560046f1`, main vertex
`bbs_3mf.cpp:3772-3782` and ObjectImporter vertex `:5485-5495` both zero-default
missing x/y/z. Triangle defaults remain deferred.

Input-only committed pair:
- `control.3mf`: `adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`
- `positive.3mf`: `9da5e1e074ee10e72d2e843c273f1282802b148421fe0add38e03d638c975780`

Only `3D/Objects/overhang.stl_1.model` changes: one resource vertex x
attribute is removed. Full before/after entry SHA manifest is committed. No
Native outputs are committed.

The test uses `OrcaRunner::from_env()` else-return, runs fresh Native at runtime
on positive/repeat/control, then invokes validated API and real bare CLI. Control
is public-tested first. Every public case asserts successful API (`unwrap_or_else`),
CLI success, raw API==CLI, and the unchanged whole generator-only Native/API/CLI
strict comparator. There is no `is_err`, `assert_ne`, expected-error branch,
or divergent-pass path. Current HEAD must fail when positive API rejects missing
`@x`; the failure is the target-equality RED. M73/motion/arcs/statistics/footer
remain in the comparator.
