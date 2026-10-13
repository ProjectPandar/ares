# Slice 3: six print identity cases accounted in the matrix

Approved SPEC0efc202d4; independent RED3c7d43699 on main96328f269.
This is a test-only accounting integration. No core/CLI production,
comparator, inventory or acceptance assertion changes. s5/s6 remain unstarted.

## Source boundary and final change

PrintConfig.cpp::PrintConfigDef::init_fff_params:5405-5417 defines the paired
print ID/variant vectors. DynamicPrintConfig::get_index_for_extruder:9004-9055
and update_values_to_printer_extruders:9634-9683 select/materialize variants;
PrintApply.cpp::Print::apply:1165-1172 applies them. GCode.cpp:5591-5644 emits
the independently asserted materialized configuration. No identity policy
or new slicing algorithm was invented.

The exact existing six requests/expectations now live in
option_coverage/print_extruder_identity/cases.rs. Standalone tests and the
matrix use the same Native/API comparison body. Its2 assert! and9 assert_eq!
checks remain, including input ZIP preservation, Native materialized view,
input bytes after slicing and complete generator-only output equality.
The optional-runner behavior remains in the standalone entry; explicit
matrix execution cannot claim credit without the supplied live runner.

Only print_extruder_id receives the generator/executor hook. It replaces
one None baseline with reversed/ordered/default/repeated/short/unmatched
requests. The executor records the bounded domain and each complete
comparison's paths/request/expected values. Receipts publish through a local
NamedTempFile rename because the sweep and its regression can write the same
case concurrently; no shared artifact API changed. Existing subset filtering
still clears completeness. Arbitrary IDs/vector shapes, other printer/variant
contexts, master/physical identity and production changes remain deferred.

## Final paired results, not a full option resweep

| Mode | Fresh root/cache/target | Tests | Matrix row |
|---|---|---|---|
| official | /tmp/worker2-s3-green-final-official.CIoeQp | 7/7 PASS, exit0 | PASS,6 generated/6 compared/0 rejected |
| seeded | /tmp/worker2-s3-green-final-seeded.rcCSDQ | 7/7 PASS, exit0 | PASS,6 generated/6 compared/0 rejected |

Outer taskset CPU13/14 respectively; jobs2/threads2. Selection is the six
existing print_process_identity tests plus the public matrix regression.
Driver:/tmp/worker2-s3-green-final-leg.sh, using the existing directed control
wrapper. All START/END posted to inbox. Every final run had independent fresh
Native cache, artifact root and target. No full/workspace was run.

The RED regression is byte-for-byte unchanged from3c7d43699. Both final
external summaries expose the finite six-case scope, and all12 matrix
comparison receipts resolve to retained input/Native/API/settings artifacts.
An independent offline reread of24 streams (six standalone plus six matrix
per mode) confirms full equality using only the existing
cli_retained_input.rs:54 generator-line normalization. Requested ID/variant
vectors match the retained3MF config. Final source SHA values match the
snapshot taken before the final runs; no source changed during them.

Reproducible reread:/tmp/worker2-s3-verify-final.py; external SHA/receipt report:
/tmp/worker2-s3-green-final-review-evidence.json. Source snapshot:
/tmp/worker2-s3-final-source-snapshot.json. No Native products committed.

## Engineering checks and retained initial failure

Final engineering root:/tmp/worker2-s3-engineering.vsxztg

- cargo clippy --workspace --all-targets --all-features -- -D warnings:0.
- cargo fmt --all -- --check:0.
- cargo check -p ares-core -p ares-wasm --target wasm32-unknown-unknown --all-features:0.

Checks ran with outer taskset13,14, jobs2 and clean standard env, no Native.
The initial GREEN pair was also7/7 each, but its first clippy failed the
repository's five-argument limit after runner made the old helper six args.
Retained failure root:/tmp/worker2-s3-engineering.4lCKSR.
The final helper takes the shared Case directly; the redundant wrapper was
removed, with no lint suppression or changed assertion. Engineering checks
then passed and both Native legs were rerun with fresh roots on final code.

Exit: submit separate s3 REVIEW-REQUEST and wait; no push or heavy-gate claim.
The unchanged historical audit still owns all other domains/ND/SLA findings.
Any coordinator-required full/workspace or merge carry remains pending GO.
