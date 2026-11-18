"""Run with Python 3 from the repository root, passing a new evidence directory."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile


SUITE = "ares-cli::orca_parity"
FUZZY = "object_emission_cases::fuzzy_allwalls_rng_order::"
SEEDED = FUZZY + "fuzzy_allwalls_seeded_oracle_ordered_bytes"
OFFICIAL = FUZZY + "official_fuzzy_allwalls_oracle_is_explicitly_unstable"
BOOKKEEPING = (
    "option_coverage::bookkeeping_export::bookkeeping_short_vectors_pad_to_filament_count"
)
ORDINARY = (
    "runner::profile_entry::plater_stl_session::klipper_time_estimator::"
    "klipper_stl_session_processor_times_match_native"
)
SHORT_LINE = (
    "option_coverage::initialized_short_line::"
    "initialized_short_line_matches_complete_internal_fill_output"
)
SWEEPS = {
    "smoke::printer_sweep": "ARES_PARITY_SWEEP",
    "stock_presets::sweep::tests::stock_printer_sweep": "ARES_PARITY_STOCK_SWEEP",
}


def main(evidence):
    evidence.mkdir(parents=True)
    env = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("ARES_", "ORCA_", "NEXTEST_"))
    }
    errors = []

    def check(ok, message):
        if not ok:
            errors.append(message)

    def nextest(label, command, *args, extra=None):
        invocation = [
            "cargo", "nextest", command, "--locked", "-p", "ares-cli",
            "--test", "orca_parity", *args,
        ]
        with (evidence / f"{label}.stdout").open("w") as out, (
            evidence / f"{label}.stderr"
        ).open("w") as err:
            result = subprocess.run(invocation, env=env | (extra or {}), stdout=out, stderr=err)
        (evidence / f"{label}.exit").write_text(f"{result.returncode}\n")
        return result.returncode

    with tempfile.TemporaryDirectory() as scratch:
        # Existing, non-executable file: none of the rejection cases may launch Native.
        supplier = Path(scratch) / "not-a-slicer"
        supplier.touch()
        required = dict(
            ARES_REQUIRE_ORCA="1",
            ARES_ORCA_BIN=str(supplier),
            ARES_ORCA_NATIVE_BIN=str(supplier),
            ARES_ORCA_RESOURCES_DIR=scratch,
            ARES_PARITY_ARTIFACT_ROOT=scratch,
        )
        official_env = {"ARES_PARITY_LANE": "official"}
        seeded_env = {"ARES_PARITY_LANE": "seeded", "ARES_PARITY_SEEDED_REFERENCE": "1"}
        for label, test, settings, diagnostic in [
            ("seeded-in-official", SEEDED, {"ARES_PARITY_LANE": "official"},
             "expected ARES_PARITY_LANE=seeded, got official"),
            ("direct-in-official", BOOKKEEPING, {"ARES_PARITY_LANE": "official"},
             "expected ARES_PARITY_LANE=seeded, got official"),
            ("official-in-seeded", OFFICIAL,
             {"ARES_PARITY_LANE": "seeded", "ARES_PARITY_SEEDED_REFERENCE": "1"},
             "expected ARES_PARITY_LANE=official, got seeded"),
            ("missing-lane", SEEDED, {}, "missing ARES_PARITY_LANE"),
            ("invalid-lane", SEEDED, {"ARES_PARITY_LANE": ""}, "invalid ARES_PARITY_LANE"),
            ("contradictory-lane", SEEDED, {"ARES_PARITY_LANE": "seeded"},
             "ARES_PARITY_LANE=seeded requires ARES_PARITY_SEEDED_REFERENCE=1"),
            ("ordinary-without-switch", ORDINARY, official_env, "missing ARES_PROCGCODE_BIN"),
            ("seeded-without-switch", SHORT_LINE,
             seeded_env | {"ORCA_REPLAY_SEED": "invalid-replay-seed"}, "invalid-replay-seed"),
            *((flag.lower(), test, official_env, f"requires {flag}=1")
              for test, flag in SWEEPS.items()),
        ]:
            status = nextest(
                label, "run", "--profile", "default", "--ignore-default-filter",
                "--retries", "0", "--failure-output", "immediate", "-E", f"test(={test})",
                extra=required | settings,
            )
            log = (evidence / f"{label}.stderr").read_text() + (
                evidence / f"{label}.stdout"
            ).read_text()
            check(status == 100 and diagnostic in log and test in log,
                  f"{label}: expected test failure ({diagnostic}), got exit {status}")

        status = nextest(
            "optional-missing-native", "run", "--profile", "default", "--success-output",
            "immediate", "-E", f"test(={ORDINARY})",
            extra={"ARES_ORCA_BIN": str(supplier), "ARES_ORCA_RESOURCES_DIR": scratch,
                   "ARES_PARITY_ARTIFACT_ROOT": scratch},
        )
        log = (evidence / "optional-missing-native.stderr").read_text()
        check(status == 0 and "SKIPPED: ARES_PROCGCODE_BIN is unset" in log,
              f"optional case must report absence of its Native supplier, got exit {status}")

    def selected(label, profile, *args):
        status = nextest(label, "list", "--profile", profile, "--message-format", "json", *args)
        if status != 0:
            errors.append(f"{label}: nextest list exited {status}")
            return None
        tests = json.loads((evidence / f"{label}.stdout").read_text())["rust-suites"][SUITE]["testcases"]
        return {name for name, test in tests.items() if test["filter-match"]["status"] == "matches"}

    default = selected("default-list", "default")
    official = selected("official-list", "parity-official")
    seeded = selected("seeded-list", "parity-seeded")
    if default is not None and official is not None and seeded is not None:
        removed = default - official
        check(len(removed) == 25 and {SEEDED, BOOKKEEPING, *SWEEPS} <= removed,
              f"official must exclude 23 seeded-only tests and two sweeps, got {sorted(removed)}")
        check(default - seeded == {OFFICIAL, *SWEEPS},
              "seeded must exclude only the official instability test and two sweeps")
        check(default - SWEEPS.keys() == official | seeded,
              "only the two sweeps may be excluded from both lanes")
        expression = " | ".join(f"test(={name})" for name in (SEEDED, OFFICIAL, *SWEEPS))
        check(selected("official-scoped-list", "parity-official", "-E", expression) == {OFFICIAL},
              "official profile must intersect the task filter")
        check(selected("seeded-scoped-list", "parity-seeded", "-E", expression) == {SEEDED},
              "seeded profile must intersect the task filter")

    for profile in ("parity-official", "parity-seeded"):
        label = f"{profile}-sweeps"
        expression = " | ".join(f"test(={name})" for name in SWEEPS)
        status = nextest(label, "run", "--profile", profile, "--no-tests=pass", "-E", expression)
        log = (evidence / f"{label}.stderr").read_text()
        check(status == 0 and "0 tests run: 0 passed" in log and "PASS " not in log,
              f"{profile} must not execute either sweep, got exit {status}")

    if errors:
        sys.exit("\n".join(errors))
    print("Lane rejection and nextest selection checks passed")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(f"usage: {sys.argv[0]} NEW_EVIDENCE_DIR")
    main(Path(sys.argv[1]))
