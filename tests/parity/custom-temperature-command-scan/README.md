# Custom temperature-command scanner public RED

Upstream: `GCode.cpp:528-589`, `4025-4054`, `4058-4090`.
SPEC: `docs/architecture/2026-10-24-custom-temperature-command-scan-spec.md`.

All projects derive from the stock Ender-3 `orca_cli_ender3.3mf` project. Disable
prime tower for every variant. Only the editable machine-start G-code differs:

| Input | SHA256 | Derivation |
|---|---|---|
| `spaced-control.3mf` | `a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae` | Original stock start G-code, with spaced M140/M190/M104/M109 |
| `compact-bed.3mf` | `e78d5230a495fd486ee289496d1fd7ccd6f2e903a33efa3f3afb423a415890bf` | Delete the command/S separator for both M140 and M190 only |
| `compact-nozzle.3mf` | `9a41b0b7d33f19f322f45214dff1f182c4bd1479ed0419ab5e7da10f02280fed` | Delete the command/S separator for M104 and M109 only |
| `comment-control.3mf` | `44a5499fc36ebc53b457beccc7b36598d102875e4a6c231e64791464854e6b82` | Remove executable temperature commands; include them only in a full-line and trailing comment, retaining other stock start commands |

GUI provenance: Printer Settings → Machine G-code → Machine start G-code,
`OrcaSlicer/src/slic3r/GUI/Tab.cpp:4593-4604` (option at4600). No custom API or
geometry manipulation. Archive entry dates fixed to1980-01-01.

Native-first public seam: `native_stream_case::check` qualifies two fresh Native
slices and their generator-normalized byte equality, runs callback assertions
that compact commands survive rendering and suppress generated startup heating
(or comments do not), then compares the full Native stream with public core API
and CLI output using the unchanged generator-only comparator. No checked-in
Native stream or output mask. Callback observations are not acceptance constants.

Failure classes documented before implementation: missing compact recognition,
accidental prefix match M1040, signed/whitespace `strtol` mismatch, changed M-only
presence when S absent, treating G10 without parsed S as temperature, scanning
comments or crossing CR/LF incorrectly. This RED pins compact bed/nozzle and
spaced/comment public cases only, not a claim of coverage for all lexical edges.

Authorized future port must preserve literal upstream semantics:
- `strtol` skips leading whitespace, accepts sign and parses integer prefix;
- C++ conditional precedence is `(endptr!=nullptr && endptr!=ptr && is_gcode)
  ? mgcode==10 : (mgcode==dont_wait || mgcode==and_wait)`;
- matching M counts without S; G10 counts only after S parses;
- comment and CR/LF scanning follows source progression exactly.
No GREEN accompanies this input/test commit. The root parity registry is not
edited; the test module is registered as a separate block in `runner.rs`, which
remains below400LOC without removing blank separators or reordering other modules.
