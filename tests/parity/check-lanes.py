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
            ("missing-case-flag", BOOKKEEPING,
             {"ARES_PARITY_LANE": "seeded", "ARES_PARITY_SEEDED_REFERENCE": "1"},
             "requires ARES_PARITY_BOOKKEEPING_REFERENCE=1"),
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
        check(len(removed) == 23 and {SEEDED, BOOKKEEPING} <= removed,
              f"official must exclude 23 seeded-only tests, got {sorted(removed)}")
        check(default - seeded == {OFFICIAL}, "seeded must exclude only the official instability test")
        check(default == official | seeded, "a test was added or excluded from both lanes")
        expression = f"test(={SEEDED}) | test(={OFFICIAL})"
        check(selected("official-scoped-list", "parity-official", "-E", expression) == {OFFICIAL},
              "official profile must intersect the task filter")
        check(selected("seeded-scoped-list", "parity-seeded", "-E", expression) == {SEEDED},
              "seeded profile must intersect the task filter")

    if errors:
        sys.exit("\n".join(errors))
    print("Lane rejection and nextest selection checks passed")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(f"usage: {sys.argv[0]} NEW_EVIDENCE_DIR")
    main(Path(sys.argv[1]))
