# Source-owned processor seam replay inputs

Owner: OrcaSlicer 2.4.2 `8500fcdcc`, GCodeProcessor.cpp::process_G1,
TimeMachine::calculate_time and GCodeProcessor.hpp::SeamsDetector.

The five committed 3MFs are slicing inputs. classification.3mf embeds the
handwritten classification-injected.gcode in machine_start_gcode; the other
archive entries retain the anchor geometry. SHA256SUMS records input identities.
No Native output is committed.

CLI tests in runner/processor_migration.rs obtain complete runtime Native output
through OrcaRunner and ARES_ORCA_CACHE_DIR, invert only generated M73 progress
and normal-mode duration fields, then call public process_gcode(post_process=true).
All remaining bytes, including movement/role/F-only/M204/configuration/whitespace,
are retained. Whole replay output must equal unmodified Native bytes, including
the generator line. Runtime input/Native/Ares artifacts are retained by the runner.

Five seam cases preserve their original initial limits; the independent positive
envelope case uses actual embedded machine/flavor/load configuration. Required
Native availability and optional visible SKIPPED follow the shared slice-A policy.
See docs/architecture/native-artifact-cleanup-processor-spec.md for the exact
12-test migration and the two input-less DEFERRED envelope diagnostics.

The obsolete reproduce.py external historical patch/source-copy workflow is
removed. Historical evidence established the source-owned seam/time-cache fix;
it is not a prerequisite or substitute for today's runtime Native comparison.
