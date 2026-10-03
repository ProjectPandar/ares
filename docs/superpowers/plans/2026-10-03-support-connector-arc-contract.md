# Support connector arc implementation plan

Source: FillBase.cpp connector phases and libslic3r.h epsilon macros.
Destination: existing fill/base_support children; architecture: ARD0093.

1. Retain the witnessed current-endpoint/equal-X correction from the held
   Tree output investigation; complete the independently found epsilon gap.
2. Verify a staged source-only snapshot on origin/main with existing Fill and
   geometry behavior tests, fmt and clippy. No newly rejecting support state.
3. Commit/push this independent prerequisite with source-cited documents.
   Keep coupled region/Normal/Tree installation together until final gates.

The epsilon mismatch is proven by source comparison, not by an existing
threshold-sensitive RED fixture. Final combined output regression evidence
will be recorded in the Tree plan without overstating TDD coverage.

Independent verification on source snapshot
`1168bda12f017eebcaa1ef26bb9c2d0496f807db` at
`/tmp/ares-connector-snapshot.TgUuK1`, based on origin/main94d33b59:

- `cargo nextest run -p ares-core -E 'test(fill::) | test(geometry::)'`:
  1408 PASS, 33 slow, 5664 excluded, 243.152s;
  `/tmp/ares-connector-snapshot-tests.log`, exit0.
- `cargo clippy -p ares-core --all-targets`: exit0, 5m41s;
  `/tmp/ares-connector-snapshot-clippy.log`. Existing lint warnings are
  252 lib/268 lib-test; no dead_code, no new suppression.
- `cargo fmt --all --check` and staged diff check: exit0.
- Independent six-axis review accepted the ten-file source-closed prerequisite
  conditional on these actual terminal results, now collected.

Combined runtime gates remain separate: the expanded official control found
an inherited-second-filament output difference. It does not invalidate this
independently compiled connector prerequisite or authorize runtime submission.
