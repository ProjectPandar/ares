"""Run an externally supplied, unmodified Moonraker parser on public exports."""

import importlib.util
import json
import pathlib
import sys

parser, pillow, output = map(pathlib.Path, sys.argv[1:4])
sys.path.insert(0, str(pillow))
import PIL

spec = importlib.util.spec_from_file_location("moonraker_metadata", parser)
metadata = importlib.util.module_from_spec(spec)
spec.loader.exec_module(metadata)
labels = ("native", "api", "cli", "wasm")
exports = {
    label: metadata.extract_metadata(path, [])
    for label, path in zip(labels, sys.argv[4:], strict=True)
}
keys = ("estimated_time", "filament_total", "filament_weight_total", "filament_weights")
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
for label, record in exports.items():
    expected_name = "OrcaSlicer" if label == "native" else "Ares"
    assert record.get("slicer") == expected_name, (label, "slicer", record.get("slicer"))
    assert record.get("slicer_version") == "2.5.0", (label, "slicer_version")
    assert all(key in record and record[key] is not None for key in keys), (
        label, "missing duration or filament metadata", values[label]
    )
    assert all(record[key] > 0 for key in keys[:3]), (label, "empty scalar metadata")
    assert record["filament_weights"] and all(
        weight > 0 for weight in record["filament_weights"]
    ), (label, "empty filament weights")
    assert values[label] == values["native"], (label, "Native metadata mismatch")
