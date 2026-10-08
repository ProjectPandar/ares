# Stage1 object range layer-height runtime RED

Baseline9fa8e9e5c (includes dbcd52d5a seam-current-layer-height behavior), no
production Rust change. Approved spec9fa8e9e5c/e12b5ff3; mixed Some/None
clarification separately confirmed by coordinator before any implementation.

Independent input derives from existing seam-overhang fixture SHA
699e9effa47a72c0de5292532aed8b1966813705f55fd0b4b22fdca661b04ee6.
Only `Metadata/layer_config_ranges.xml` was appended: object-order1, interval
0.3..0.6mm at0.1mm. Every other archive member payload is unchanged. Result
fixture SHA fc938f743bed802de2691134f232015e715d525fc87f9f4b33ce6a51785b9b5f.
Nominal height0.2, first0.3, nearest seam, legal nozzle-height range0.08..0.36.

Native existing admitted seeded2.4.2 binary (no rebuild) exported successfully:
exit0,26578 bytes, raw SHA0b6607dfc4587a7935af263a5a48b2276b88880d83b9caac3a1104fdb3342cbf.
Native actual layers: (print_z,height)=(0.3,0.3),(0.4,0.1),(0.5,0.1),(0.6,0.1).
This proves range consumption while global layer_height remains0.2 and also
exercises seam overhang with nonuniform actual thickness. Full raw Native G-code
and input/producer/command/source receipt are committed beside the fixture.

An initial shell wrapper preflight did not launch Native because required
ORCA_REPLAY_CPU was omitted; its stdout/stderr/exit were preserved separately.
Actual independent Native command ran once on CPU11/seed0; no failed Native
slice was retried. `/tmp/worker3-layer-range-native-red/` holds those receipts.

Then wrote integration test at the existing public byte-slicing seam, checking
input identity, Native layer schedule, and eventual entire generator-line-only
byte comparison. Runtime RED through cargo nextest:
- `/tmp/worker3-layer-range-runtime-red.log/.exit`, exit100;
- run2929f3f7-8af6-465c-8364-a9b68c151954,1 failed/661 filtered/0 ignored;
- current Ares rejects with `unsupported project feature: layer_height`;
- artifact `/tmp/worker3-layer-range-red-artifacts/object-range-height-Nhhfln/`;
- independent target `/tmp/worker3-layer-range-target`, fresh native cache,
  parity threads16/CARGO_BUILD_JOBS8.

The first test compilation failed due ExportedCase private fields when placed
outside runner's module. This is recorded in `/tmp/worker3-layer-range-red.log`
(exit101), not called runtime RED; moved test into existing runner child-module
scope without changing production visibility, then obtained the above runtime
failure. No acceptance comparison was relaxed.

Before production, older gate/preflight tests are updated to expect explicit
height capability acceptance but retain raw-range preflight rejection. A separate
prewritten mixed-height/region-only input keeps UnsupportedProjectFeature
("layer_config_ranges"): source's missing layer_height dereference is UB, not a
portable Native output oracle, so this control must not launch Native.

No GREEN or full gates claimed at this commit. Painted-profile and ZAA gates
remain unchanged; implementation follows this independent runtime RED.
