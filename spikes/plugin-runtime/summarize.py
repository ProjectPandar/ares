import json
import statistics
import sys
from pathlib import Path

root = Path(sys.argv[1])


def read_lines(name):
    return [json.loads(line) for line in (root / name).read_text().splitlines()]


def median(rows, key):
    values = [row[key] for row in rows if row[key] is not None]
    return statistics.median(values) if values else None


def fmt(value):
    return f"{value:.3f}" if value is not None else "—"


def stable(rows, key):
    values = {row[key] for row in rows}
    assert len(values) == 1, key
    return values.pop()


rows = {}
incomplete = []
for kind in ("staggered", "subdivision"):
    for platform in ("native", "browser"):
        for backend in ("direct", "interpreter", "jit" if platform == "native" else "browser"):
            for engine in ("rust", "quickjs", "boa"):
                prefix = f"{platform}-{backend}-{engine}-{kind}"
                try:
                    samples = read_lines(prefix + ".jsonl")
                except (FileNotFoundError, json.JSONDecodeError):
                    samples = []
                if len(samples) != 5:
                    incomplete.append(prefix)
                    continue
                rows[platform, backend, engine, kind] = samples

print("| Platform | Placement | Plugin | Recipe | ms/layer | ms/300 | Load ms | Reload ms |")
print("| --- | --- | --- | --- | ---: | ---: | ---: | ---: |")
for (platform, backend, engine, kind), samples in rows.items():
    total = median(samples, "hook_total_ms")
    load = median(samples, "load_ms") if (backend, engine) != ("direct", "rust") else None
    print(f"| {platform} | {backend} | {engine} | {kind} | "
          f"{fmt(total / 300)} | {fmt(total)} | {fmt(load)} | {fmt(median(samples, 'reload_ms'))} |")

if incomplete:
    print("\nMissing or incomplete five-trial rows:", ", ".join(incomplete))

print("\nLayer edit digests (complete rows)")
for kind in ("staggered", "subdivision"):
    hashes = {stable(samples, "layer_sha256") for key, samples in rows.items() if key[3] == kind}
    assert len(hashes) == 1, (kind, hashes)
    print(kind, hashes.pop())

print("\nG-code digests")
for engine in ("rust", "quickjs", "boa"):
    hashes = {stable(samples, "gcode_sha256") for key, samples in rows.items() if key[2] == engine}
    assert len(hashes) == 1, (engine, hashes)
    print(engine, hashes.pop())

print("\nBuilt-in math before portable shim")
for backend in ("direct", "jit"):
    for engine in ("quickjs", "boa"):
        native = read_lines(f"native-raw-math-{backend}-{engine}.jsonl")[0]
        web_backend = "browser" if backend == "jit" else backend
        web = read_lines(f"browser-raw-math-{web_backend}-{engine}.jsonl")[0]
        print(backend, engine, native["gcode_sha256"], web["gcode_sha256"])
        print("native: ", native["gcode"].splitlines()[-1])
        print("browser:", web["gcode"].splitlines()[-1])

print("\nMetering: ms/300 without and with metering")
for platform, backend in (("native", "interpreter"), ("native", "jit"), ("browser", "interpreter")):
    base = rows[platform, backend, "rust", "staggered"]
    metered = read_lines(f"{platform}-metered-{backend}-rust-staggered.jsonl")
    assert len(metered) == 5
    assert stable(base, "layer_sha256") == stable(metered, "layer_sha256")
    print(platform, backend, fmt(median(base, "hook_total_ms")), fmt(median(metered, "hook_total_ms")))

print("\nResource probes")
for file in ("native-limits.json", "native-direct-limits.json", "browser-limits.json", "component-browser.json"):
    print(file, json.loads((root / file).read_text()))
native = json.loads((root / "native-limits.json").read_text())
web = json.loads((root / "browser-limits.json").read_text())
assert native["interpreter"] == web["interpreter"], (native, web)

print("\nFiles: median load ms and us/read+write over 10,000 operations")
for platform in ("native", "browser"):
    for mode in ("wasi", "custom"):
        result = json.loads((root / f"{platform}-files-{mode}.json").read_text())
        samples = result["runs"] if platform == "browser" else result
        times = [(row["run_ms"] / row["iterations"]) * 1000 for row in samples]
        persistence = {k: v for k, v in result.items() if k != "runs"} if platform == "browser" else {}
        print(platform, mode, fmt(median(samples, "load_ms")), fmt(statistics.median(times)), persistence)

print("\nRaw Wasm sizes in bytes")
base = (root / "baseline_bg.wasm").stat().st_size
for path in sorted(root.glob("*.wasm")):
    size = path.stat().st_size
    print(path.name, size, (size - base) if path.name.endswith("_bg.wasm") else "")

if incomplete:
    sys.exit(1)
