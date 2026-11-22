"""Run an externally supplied, unmodified Moonraker parser on public exports."""

import importlib.util
import json
import pathlib
import sys

mode, ares_version = sys.argv[1:3]
parser, pillow, output = map(pathlib.Path, sys.argv[3:6])
sys.path.insert(0, str(pillow))
import PIL

spec = importlib.util.spec_from_file_location("moonraker_metadata", parser)
metadata = importlib.util.module_from_spec(spec)
spec.loader.exec_module(metadata)
labels = ("native", "api", "cli", "wasm")
exports = {
    label: metadata.extract_metadata(path, False if mode == "legacy" else [])
    for label, path in zip(labels, sys.argv[6:], strict=True)
}
scalar_keys = ("estimated_time", "filament_total", "filament_weight_total")
# v0.9.3 metadata.py:929–953 exposes total weight but not per-filament weights.
keys = scalar_keys if mode == "legacy" else (*scalar_keys, "filament_weights")
values = {
    label: {key: record.get(key) for key in keys}
    for label, record in exports.items()
}
result = {
    "python": sys.version,
    "pillow_version": PIL.__version__,
    "pillow_module": PIL.__file__,
    "exports": exports,
    "comparison": values,
}
(output / "moonraker-metadata.json").write_text(json.dumps(result, indent=2))
print(json.dumps(values, indent=2))
noncomparable = {"size", "gcode_start_byte", "gcode_end_byte", "uuid", "modified"}
native = exports["native"]
for label, record in exports.items():
    assert record.get("slicer") == "OrcaSlicer", (label, "slicer", record.get("slicer"))
    expected_version = "2.5.0" if label == "native" else f"Ares-{ares_version}"
    assert record.get("slicer_version") == expected_version, (
        label, "slicer_version", record.get("slicer_version")
    )
    assert all(key in record and record[key] is not None for key in keys), (
        label, "missing duration or filament metadata", values[label]
    )
    assert all(record[key] > 0 for key in scalar_keys), (label, "empty scalar metadata")
    if mode == "current":
        assert record["filament_weights"] and all(
            weight > 0 for weight in record["filament_weights"]
        ), (label, "empty filament weights")
    if label != "native":
        fields = (native.keys() | record.keys()) - noncomparable - {"slicer_version"}
        differing = sorted(
            key for key in fields
            if key not in native or key not in record or record[key] != native[key]
        )
        assert not differing, (label, "Native metadata mismatch", differing)
