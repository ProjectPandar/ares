# Closest-lines Native inputs

Both projects contain normal overhang parts and disable interlocking. They are
unchanged authored inputs extracted from the interlocking prerequisite tests;
no Native output is committed.

- `two-extruder.3mf`: SHA256 `1dbedfe681a6efdd527c4f032ddfe26cfb9746e817711a5d59bc4889e5751486`.
- `single.3mf`: SHA256 `1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651`.

Native twice is stable in both official and seeded
modes. Single-volume passes; two-extruder API/CLI raw bytes agree but fail the
unchanged complete generator-only comparison at line 728.
The E2E tests retain input, native/replay launch arguments and streams, API/CLI
streams and comparison JSON so the result can be repeated and verified.
