"""Input-only wipe_inward witnesses; preserve the base mesh byte-for-byte."""
from hashlib import sha256
import json
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile

ROOT = Path(__file__).resolve().parent
BASE = ROOT.parent / "arachne-fuzzy-f1/none.3mf"
COMMON = {
    "wall_generator": "classic", "wall_sequence": "inner wall/outer wall",
    "wall_direction": "ccw", "line_width": "0.45",
    "outer_wall_line_width": "0.45", "inner_wall_line_width": "0.45",
    "initial_layer_line_width": "0.45", "layer_height": "0.2",
    "initial_layer_print_height": "0.2", "precise_outer_wall": "0",
    "sparse_infill_density": "0%", "top_shell_layers": "0", "bottom_shell_layers": "0",
    "top_shell_thickness": "0", "bottom_shell_thickness": "0",
    "only_one_wall_top": "0", "only_one_wall_first_layer": "0",
    "detect_thin_wall": "0", "detect_overhang_wall": "0", "overhang_reverse": "0",
    "unsupported_wall_last": "0", "is_infill_first": "0",
    "fuzzy_skin": "none", "gap_fill_target": "nowhere", "seam_position": "aligned",
    "seam_gap": "0", "seam_slope_type": "none", "spiral_mode": "0",
    "enable_arc_fitting": "disabled", "enable_support": "0", "enable_prime_tower": "0",
    "brim_type": "no_brim", "brim_width": "0", "skirt_loops": "0",
    "elefant_foot_compensation": "0", "wipe": ["1"], "filament_wipe": ["nil"],
    "wipe_distance": ["2"], "filament_wipe_distance": ["nil"],
    "wipe_inward_distance": "50%", "wipe_on_loops": "0", "wipe_before_external_loop": "0",
    "role_based_wipe_speed": "0", "wipe_speed": "30",
    "retract_when_changing_layer": ["1"], "filament_retract_when_changing_layer": ["nil"],
    "retraction_length": ["0.8"], "filament_retraction_length": ["nil"],
    "retract_before_wipe": ["0%"], "filament_retract_before_wipe": ["nil"],
    "use_relative_e_distances": "1",
}
CASES = [("enabled-inner-outer", "3", "1"), ("disabled-control", "3", "0"),
         ("single-wall-no-anchor", "1", "1")]

with zipfile.ZipFile(BASE) as source:
    original = {name: source.read(name) for name in source.namelist()}
xml = ET.fromstring(original["3D/Objects/cube10.stl_1.model"])
points = [tuple(float(node.attrib[a]) for a in ["x", "y", "z"])
          for node in xml.iter() if node.tag.rsplit("}", 1)[-1] == "vertex"]
bounds = [[min(point[i] for point in points), max(point[i] for point in points)] for i in range(3)]
assert bounds == [[-15, 15], [-15, 15], [-0.4, 0.4]]
manifest = {"upstream": "345efec64c7b95a05ce5cf96df0dde66a158835c",
    "source": "WipePathHelpers.cpp:21-884;GCode.cpp:668-756,8494,8729-8801,8847-8854,8903-8929",
    "base": "tests/parity/arachne-fuzzy-f1/none.3mf",
    "base_sha256": sha256(BASE.read_bytes()).hexdigest(), "unchanged_mesh_bounds_mm": bounds,
    "Native_output_embedded": False, "cases": {}}
for name, walls, enabled in CASES:
    members = original.copy()
    settings = json.loads(members["Metadata/project_settings.config"])
    overrides = COMMON | {"wall_loops": walls, "wipe_inward": enabled}
    settings.update(overrides)
    members["Metadata/project_settings.config"] = json.dumps(settings, sort_keys=True, indent=2).encode()
    assert all(data == original[member] for member, data in members.items()
               if member != "Metadata/project_settings.config")
    path = ROOT / f"{name}.3mf"
    with zipfile.ZipFile(path, "w") as out:
        for member, data in members.items():
            assert not member.endswith(".gcode")
            info = zipfile.ZipInfo(member, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            out.writestr(info, data)
    manifest["cases"][name] = {"input_sha256": sha256(path.read_bytes()).hexdigest(),
        "settings_overrides": overrides, "geometry_unchanged_from_base": True,
        "member_sha256": {member: sha256(data).hexdigest() for member, data in members.items()}}
(ROOT / "provenance.json").write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
for name, case in manifest["cases"].items():
    print(name, case["input_sha256"])
