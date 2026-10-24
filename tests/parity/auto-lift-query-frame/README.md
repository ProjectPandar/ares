# Auto-lift query-frame public parity witness

Source boundary: OrcaSlicer `GCode.cpp::needs_retraction:7604`, current-plate query translation by origin minus writer plate XY offset. See `docs/architecture/auto-lift-query-frame-spec.md` and its ARD.

`project.3mf` is the unchanged original Native-exported stock P1S 0.4-nozzle two-object ByLayer discovery input (overhang and cube-with-hole). SHA256: `f22c2132032860447369a3b8d29291230108f20ef1030cb532447cfb90ae31dd`. Do not regenerate or change settings/geometry/placement to isolate progress behavior.

Registered test:

- `case_support::auto_lift_query_frame::orca_parity_auto_lift_query_frame_p1s`: two fresh Native slices and unchanged generator-only replay; the existing `native_stream_case::check` callback asserts both Native witnesses before any public API/CLI calls. Its unchanged complete comparator enforces the fourth SPEC requirement: public spiral and progress bytes must match Native, as must the rest of the output; successful API/CLI output must be raw equal. No separate preflight test, expected-error wrapper, synthetic failure or progress/motion filter.

For each approved official/seeded mode, run this selected target with `cargo nextest run`. CPU18, build/test/runner threads1. Preserve full Native/API/CLI streams, success statuses, comparison receipts, executable identities and the fixture hash under the artifact root. A legitimate RED has successful raw-equal API/CLI calls but fails the unchanged complete-output comparator; an admission/preparation/Native-replay failure is a STOP, not RED.

Tests and fixture only: no production, comparator or shared runner changes. RED is expected before the query-translation port; only after review may GREEN alter that production boundary. No FULL/integration.
