# Motion attributes runtime oracle

Inputs are `tests/parity/orca_cli_ender3.3mf` and the five original template/config
recipes in `crates/ares-cli/tests/orca_parity/runner/adjacent_motion.rs`.
No Native RLE is committed. OrcaRunner processes each emitted G-code at runtime
using explicit `ARES_PROCGCODE_DYNAMICS_BIN`; only the producer name is changed
so Native loads the embedded configuration, preserving command bytes and line ids.

The initialized-Native diagnostic is defined by
`tools/chain-probe/procgcode-dynamics.{nix,patch}` on top of raw procgcode.
It initializes upstream's indeterminate PA to zero, keeps upstream's stale
OptionsZCorrector index across actual-speed insertions (no stable-options remap,
`docs/architecture/procgcode-dynamics-raw-options-z.md`), and aligns the DynamicPrintConfig
travel-acceleration flavor gate with the slicing PrintConfig path. It is labeled
a diagnostic, not an unmodified official processor.

Every ordered vertex identity and f32 bits for PA, acceleration, jerk, actual
feedrate and Normal move time are compared against the current Native CSV dumps.
Native/Ares RLE, input, stdout/stderr and command/producer receipts are retained
in the runtime artifact/cache directories, never copied into this directory.
`ARES_REQUIRE_ORCA=1` makes missing tools fail; optional omission prints SKIPPED.
Use the v2 Native environment with `cargo nextest run -p ares-cli --test
orca_parity -E 'test(runner::adjacent_motion)'`; cold and hot caches run the same
assertions. See the processor-adjacent cleanup spec for the complete contract.
