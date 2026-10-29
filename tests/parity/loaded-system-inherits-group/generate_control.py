#!/usr/bin/env python3
"""Derive the approved control by blanking only system JSON parent strings."""
import hashlib
import json
import re
from pathlib import Path

root = Path(__file__).resolve().parent
entries = {}
for name in ("machine.json", "process.json", "filament.json"):
    source = (root / "positive" / name).read_bytes()
    before = json.loads(source)
    assert before["from"] == "system" and before["inherits"]
    control, count = re.subn(rb'("inherits"\s*:\s*)"[^"\\]*"', rb'\1""', source)
    assert count == 1
    expected = dict(before, inherits="")
    assert json.loads(control) == expected
    (root / "control").mkdir(exist_ok=True)
    (root / "control" / name).write_bytes(control)
    for case, content in (("positive", source), ("control", control)):
        entries[f"{case}/{name}"] = hashlib.sha256(content).hexdigest()
entries["single.3mf"] = hashlib.sha256((root / "single.3mf").read_bytes()).hexdigest()
(root / "manifest.json").write_text(json.dumps({
    "approved_spec": "6c38e182fd8f22388833cc55b225c26dbb805dab",
    "upstream_pin": "8500fcdccaa10b5099ac20d252af3a7c560046f1",
    "source": "retained rKLPnw single-control plus identical loaded JSONs",
    "sha256": entries,
}, indent=2) + "\n")
