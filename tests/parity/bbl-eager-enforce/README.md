# BBL forced eager parent enforcement inputs

SPEC: `docs/architecture/2026-10-22-bbl-eager-retract-lift-enforce-spec.md`.
Source owner: pinned8500fcd `GCode.cpp:5205-5210,7676-7704`,
`GCodeWriter.cpp:652-681`. No Native G-code output is stored here.

Origin: `tests/parity/modifier-bottom-shell/classic-four.3mf`, stored Bambu
X1C0.4 Standard-Mine **customized** process, not pristine factory.
Control is an exact byte copy. Each target changes only element0 of
`retract_lift_enforce`; all other settings, geometry, transforms, modifier
bottom-shell4 and non-settings ZIP payloads are unchanged.

| Input | On surfaces | SHA256 |
|---|---|---|
|all-surfaces.3mf|All Surfaces control|438bf4c61895a7a8d51e3c61a70fa8df120a9f3a607d63a4291d359984154a7c|
|bottom-only.3mf|Bottom Only primary|f735864a4dc6d992440a7e9d9a8b5f620775cdda7ea09d53dc09e65849664bcf|
|top-only.3mf|Top Only|bd42ee78a736e7916ae8b4c549c8c52f45c6522b89214d1029d2701b9039bc77|
|top-and-bottom.3mf|Top and Bottom|792645d56b67be089d137bc95ce7b76827fbef882782774c235ddc3974ec63c3|

Four direct `native_stream_case::check` calls first replay Native twice, then
compare the whole public API and CLI outputs using the unchanged strict
generator-only checker. M73, model/total time, E/F, wipes and footer stay in
the comparison. No error-as-success, private observer-count assertions or
expected Native output constants. The SHA values freeze **inputs only**.

RED expects one whole-output passing control and three genuine complete-stream
failures per mode, never rejection or Native instability. Target panic can
precede the shared wrapper-tail metadata; independent raw equality audits do
not claim an unexecuted wrapper assertion ran. GREEN is not authorized yet.
