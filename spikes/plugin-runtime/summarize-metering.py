import json
import statistics
import sys
from pathlib import Path


def trials(path):
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert len(rows) == 5, path
    return rows


def checksum(rows):
    pairs = {(row["layer_sha256"], row["gcode_sha256"]) for row in rows}
    assert len(pairs) == 1, pairs
    return pairs.pop()


def timing(rows):
    hook = statistics.median(row["hook_total_ms"] for row in rows) / 300
    times = [hook]
    for key in ("load_ms", "reload_ms", "runner_reload_ms"):
        values = [row[key] for row in rows if row.get(key) is not None]
        times.append(statistics.median(values) if values else None)
    return " / ".join("—" if value is None else f"{value:.3f}" for value in times)


root, prior = map(Path, sys.argv[1:])
native = json.loads((root / "native-portable-limits.json").read_text())
browser = json.loads((root / "browser-portable-limits.json").read_text())
assert native == browser, (native, browser)
print("Portable probe:", native)

print("\n| Engine | Guest | Recipe | Previous plain | Current plain | Instrumented |")
print("| --- | --- | --- | ---: | ---: | ---: |")
for platform, backend, name in (("native", "jit", "Wasmtime"), ("browser", "browser", "Chrome")):
    for guest in ("rust", "quickjs"):
        for kind in ("staggered", "subdivision"):
            previous = trials(prior / f"{platform}-{backend}-{guest}-{kind}.jsonl")
            plain = trials(root / f"{platform}-plain-{guest}-{kind}.jsonl")
            metered = trials(root / f"{platform}-portable-{guest}-{kind}.jsonl")
            assert checksum(previous) == checksum(plain) == checksum(metered), (platform, guest, kind)
            print(f"| {name} | {guest} | {kind} | {timing(previous)} | {timing(plain)} | {timing(metered)} |")

print("\nVerified prior edit and G-code digests:")
for kind in ("staggered", "subdivision"):
    for guest in ("rust", "quickjs"):
        print(kind, guest, *checksum(trials(prior / f"native-jit-{guest}-{kind}.jsonl")))

print("\nRaw ares-wasm binary sizes:")
base = (root / "baseline_bg.wasm").stat().st_size
host = (root / "metering_host_bg.wasm").stat().st_size
print(f"{base:,} → {host:,} bytes (+{host - base:,})")

print("\n| Guest | Static debits | Code section bytes | Module bytes |")
print("| --- | ---: | ---: | ---: |")
for line in (root / "guest-sizes.jsonl").read_text().splitlines():
    row = json.loads(line)
    code, module = (" → ".join(f"{size:,}" for size in row[key]) for key in ("code_bytes", "module_bytes"))
    print(f"| {row['path']} | {row['segments']:,} | {code} | {module} |")
