#!/usr/bin/env bash
set -euo pipefail

# Run non-parity workspace tests for the selected crates (or all workspace crates
# when RUST_CRATES=all), excluding the orca_parity integration test binary.

RUST_CRATES="${RUST_CRATES:-all}"
TMP_ROOT="${RUNNER_TEMP:-/tmp}"

context_artifacts="$(mktemp -d "${TMP_ROOT}/configdef-context.XXXXXXXX")"
enum_artifacts="$(mktemp -d "${TMP_ROOT}/enum.XXXXXXXX")"
export ARES_CONFIGDEF_CONTEXT_ARTIFACTS="$context_artifacts"
export ARES_ENUM_ARTIFACTS="$enum_artifacts"

python3 - "$RUST_CRATES" <<'PYEOF'
import json, subprocess, sys
selected = sys.argv[1]
want = None if selected == "all" else set(selected.split())
meta = json.loads(subprocess.check_output(["cargo", "metadata", "--no-deps", "--format-version", "1"], text=True))
other_pkgs = []
cli_args = ["-p", "ares-cli"]
for pkg in meta["packages"]:
    name = pkg["name"]
    if want is not None and name not in want:
        continue
    if name != "ares-cli":
        other_pkgs.extend(["-p", name])
    else:
        kinds = {k for t in pkg["targets"] for k in t["kind"]}
        if "lib" in kinds:
            cli_args.append("--lib")
        if "bin" in kinds:
            cli_args.append("--bins")
        for target in pkg["targets"]:
            if "test" in target["kind"] and target["name"] != "orca_parity":
                cli_args.extend(["--test", target["name"]])
if other_pkgs:
    subprocess.check_call(["cargo", "nextest", "run", "--no-fail-fast", *other_pkgs])
if len(cli_args) > 2:
    subprocess.check_call(["cargo", "nextest", "run", "--no-fail-fast", *cli_args])
PYEOF
