# TiQ2 ToolOrdering localization probe

Read-only instrumentation of upstream `Print::process`'s wipe-tower gate
and `ToolOrdering::collect_extruder_statistics` (Print.cpp:2410-2436;
GCode/ToolOrdering.cpp:921-953), plus `DynamicPrintConfig::normalize_fdm_2`
entry/exit (PrintConfig.cpp:8688-8738). No geometry/order predicates are changed.

First compare the Native `PRINT_GATE` and `STATISTICS prime` records to
Ares's `type2_requested`/`statistics::collect` inputs. Then compare
`all`, `first`, `last` and the ordered first-layer `extruders` before
interpreting front-layer tower/partition differences. IDs are zero-based.
There may be repeated records from constructor and sort/build passes;
retain their order and do not conflate them.

```sh
out=$(mktemp -d)
probe=$(nix-build tools/orca-probes/tiq2_order.nix --no-out-link | tail -n 1)
sha256sum tests/parity/validation/tiq2-type2-enabled.3mf "$probe/bin/orca-slicer"
ORCA_REPLAY_BIN="$probe/bin/orca-slicer" ORCA_REPLAY_SEED=0 \
  ORCA_REPLAY_CPU="$(awk '/Cpus_allowed_list/{split($2,a,/[-,]/);print a[1]}' /proc/self/status)" \
  ORCA_DUMP_TIQ2_ORDER="$out/native-order.txt" \
  scripts/orca-seeded-parity.sh --slice 0 \
  --outputdir "$out" tests/parity/validation/tiq2-type2-enabled.3mf
```

Expected input SHA:
`0435346ed81fff32acdae7a3a3f2c9e880f4f1efe37d99d3427833836b901942`.
Use a new output directory/log for every run. Record the resolved Nix
source/build identity. The expression reconstructs the exact seeded oracle
derivation before adding instrumentation. Compare against the seed-replay
oracle, not across builds against the AppImage. Require complete generated-by-only
byte equality before accepting any state record as a core-test expectation.

Validated seed-replay probe
`/nix/store/5v0gz2rkj0pb5yifwdvcjnilhg3azabl-orca-slicer-seed-replay-2.4.2`
equals oracle `1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv` at seed=0 on a single CPU:
816585 normalized bytes, SHA256
`8d6aec0a422bb8e7bf8de622ec2630a5e46f14aaf8d9ddbe0a3486ad0b18dda4`.
The second normalization call enters objects=1/used=1/enable=1 and exits
enable=0/changed=enable_prime_tower; the first objects=0/used=0 call preserves
enable=1. The Print gate then records enable=0/has_tower=0. The footer still
records enable=1 and must not be mistaken for the effective Print config.

Earlier stock-Nix probes `wqb1jsg...`/`cib3gbck...` and their AppImage
comparison are quarantined cross-build evidence; do not use them as oracle
expectations. Ordered statistics passes must never be merged.
