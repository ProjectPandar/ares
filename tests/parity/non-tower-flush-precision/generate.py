#!/usr/bin/env python3
"""Derive support-tool fixtures from stock X1C half-scale overhang projects.

Export tests/parity/overhang.stl at 0.5x with the v2.5.0 X1C 0.4 mm,
0.20 mm Standard, and Bambu PLA Basic presets. Pass the one-slot and
identical two-PLA-slot exports, in that order. The exports are only inputs;
this script writes deterministic synthetic slicer inputs, not sliced output.
"""

import argparse
import json
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

SETTINGS = "Metadata/project_settings.config"


def write_project(source, destination, slots, **overrides):
    with ZipFile(source) as original:
        config = json.loads(original.read(SETTINGS))
        assert len(config["filament_colour"]) == slots
        if slots == 2:
            assert config["flush_multiplier"] == ["0.3"]
            assert config["flush_volumes_matrix"] == ["0", "458", "229", "0"]
            config["filament_colour"] = ["#D03020", "#2266CC"]
        config.update(
            enable_support="1",
            support_type="normal(auto)",
            support_style="default",
            farthest_point_timelapse="0",
            enable_prime_tower="0",
            support_filament="0",
            support_interface_filament="0",
        )
        config.update(overrides)
        with ZipFile(destination, "w") as generated:
            for name in sorted(original.namelist()):
                data = (
                    json.dumps(config, sort_keys=True, separators=(",", ":")).encode()
                    if name == SETTINGS
                    else original.read(name)
                )
                generated.writestr(ZipInfo(name), data, compress_type=ZIP_DEFLATED)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("single_stock", type=Path)
    parser.add_argument("dual_stock", type=Path)
    args = parser.parse_args()
    output = Path(__file__).resolve().parent
    write_project(args.single_stock, output / "single.3mf", 1)
    write_project(args.dual_stock, output / "support.3mf", 2, support_filament="2")
    write_project(
        args.dual_stock, output / "interface.3mf", 2, support_interface_filament="2"
    )


if __name__ == "__main__":
    main()
