"""Input-only Arachne deferral witnesses from frozen Classic geometry; no output."""
from hashlib import sha256
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / "classic-unsupported-wall-last"
SOURCES = {
    "unattached-ledge": ("unattached-ledge", "1"),
    "disabled-control": ("unattached-ledge", "0"),
    "attached-stack": ("attached-stack", "1"),
}
KEY = "Metadata/project_settings.config"
original = json.loads((SOURCE / "provenance.json").read_text())
manifest = {
    "upstream": "345efec64c7b95a05ce5cf96df0dde66a158835c",
    "source": "PerimeterGenerator.cpp:1391-1455,3093-3111;GCode.cpp:8889-8933",
    "derivation": "Frozen Classic geometry, Arachne selection; disabled derives from the enabled ledge and differs only by unsupported_wall_last.",
    "cases": {},
}
outputs = {}
for label, (source_label, enabled) in SOURCES.items():
    source = SOURCE / f"{source_label}.3mf"
    source_hash = sha256(source.read_bytes()).hexdigest()
    assert source_hash == original["cases"][source_label]["input_sha256"]
    with zipfile.ZipFile(source) as archive:
        members = {name: archive.read(name) for name in archive.namelist()}
    before = members.copy()
    settings = json.loads(members[KEY])
    assert settings["wall_generator"] == "classic" and settings["wipe_inward"] == "0"
    assert settings["use_relative_e_distances"] == "1"
    changes = {"wall_generator": "arachne", "unsupported_wall_last": enabled}
    settings.update(changes)
    members[KEY] = json.dumps(settings, sort_keys=True, indent=2).encode()
    assert all(members[name] == data for name, data in before.items() if name != KEY)
    path = ROOT / f"{label}.3mf"
    with zipfile.ZipFile(path, "w") as archive:
        for name, data in members.items():
            assert not name.endswith(".gcode")
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    outputs[label] = members
    manifest["cases"][label] = {
        "input_sha256": sha256(path.read_bytes()).hexdigest(),
        "source_input": f"tests/parity/classic-unsupported-wall-last/{source_label}.3mf",
        "source_input_sha256": source_hash,
        "geometry": original["cases"][source_label]["geometry"],
        "settings_overrides": changes,
        "wipe_inward": "0",
        "all_non_settings_members_identical_to_source": True,
        "member_sha256": {name: sha256(data).hexdigest() for name, data in members.items()},
        "Native_output_embedded": False,
    }
assert all(outputs["unattached-ledge"][name] == outputs["disabled-control"][name]
           for name in outputs["unattached-ledge"] if name != KEY)
a, b = (json.loads(outputs[label][KEY]) for label in ["unattached-ledge", "disabled-control"])
assert {key for key in a if a[key] != b[key]} == {"unsupported_wall_last"}
(ROOT / "provenance.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
for label, case in manifest["cases"].items():
    print(label, case["input_sha256"])
