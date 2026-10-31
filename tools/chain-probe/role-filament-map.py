"""Read the pinned Native role map via the explicit ARES_GDB_BIN launcher.

Default mode takes a single-filament, no-flush project and validates the map
against the same-run footer. ARES_ROLE_ORACLE_MODE=process instead replays
G-code boundary fixtures using the same validated map reader, with explicit
null footer_check when role totals and footer totals have different scopes.
No inferior data/register writes or Native rebuild are involved.
"""

import hashlib
import json
import os
from pathlib import Path
import struct

import gdb


ROOT = Path(os.environ["ARES_ROLE_ORACLE_OUTPUT"])
BINARY = Path(os.environ["ARES_PROCGCODE_NATIVE_BIN"])
SHA = os.environ["ARES_PROCGCODE_NATIVE_SHA256"]
assert os.environ["ARES_PROCGCODE_UPSTREAM_REV"] == "345efec64c7b95a05ce5cf96df0dde66a158835c"
assert hashlib.sha256(BINARY.read_bytes()).hexdigest() == SHA
ROOT.mkdir(parents=True)
SOURCE = Path(os.environ["ARES_ROLE_ORACLE_INPUT"]).resolve()
MODE = os.environ.get("ARES_ROLE_ORACLE_MODE", "slice")
assert MODE in ("slice", "process")
# ExtrusionEntity.hpp:20-43, not libvgcode's different role ordinal order.
ROLES = [
    "None", "Perimeter", "ExternalPerimeter", "OverhangPerimeter",
    "InternalInfill", "SolidInfill", "TopSolidInfill", "BottomSurface",
    "Ironing", "BridgeInfill", "InternalBridgeInfill", "GapFill", "Skirt",
    "Brim", "SupportMaterial", "SupportMaterialInterface", "SupportTransition",
    "WipeTower", "Custom", "Mixed",
]
snapshots = []
printer_dialects = []


def memory(pointer, size):
    return gdb.selected_inferior().read_memory(pointer, size).tobytes()


def u64(pointer):
    return struct.unpack("<Q", memory(pointer, 8))[0]


def read_map(used):
    # Pinned ELF process_role_cache disassembly:
    # +0x1a loads cache at UsedFilaments+0x130;
    # +0xb6 addresses map+0x138; +0xbd loads root at +0x148;
    # +0xe3 reads uint8_t role at node+0x20;
    # +0x155/+0x176 add pair doubles at node+0x28/+0x30.
    # libstdc++ _Rb_tree header: parent/root +16, node_count +40.
    count = u64(used + 0x160)
    seen = set()
    rows = []

    def visit(node):
        if node == 0:
            return
        assert node not in seen
        seen.add(node)
        assert len(seen) <= count
        visit(u64(node + 16))
        role = memory(node + 32, 1)[0]
        values = memory(node + 40, 16)
        meters, grams = struct.unpack("<dd", values)
        rows.append({
            "role": ROLES[role], "native_role": role,
            "meters": meters, "grams": grams,
            "bits": list(struct.unpack("<QQ", values)),
        })
        visit(u64(node + 24))

    visit(u64(used + 0x148))
    keys = [row["native_role"] for row in rows]
    assert len(rows) == count and keys == sorted(set(keys))
    assert struct.unpack("<d", memory(used + 0x130, 8))[0] == 0.0
    return rows


class Finalized(gdb.FinishBreakpoint):
    def __init__(self, used):
        super().__init__(internal=True)
        self.used = used

    def stop(self):
        snapshots.append(read_map(self.used))
        return False


class Finalize(gdb.Breakpoint):
    def stop(self):
        flag = int(gdb.parse_and_eval(
            "(unsigned char*)&'Slic3r::GCodeProcessor::s_IsBBLPrinter'"
        ))
        printer_dialects.append(bool(memory(flag, 1)[0]))
        Finalized(int(gdb.parse_and_eval("$rdi")))
        return False


gdb.execute("set pagination off")
gdb.execute("set confirm off")
gdb.execute("set print thread-events off")
gdb.execute("file " + str(BINARY))
gdb.execute(
    "set environment LD_LIBRARY_PATH " + str(BINARY.parent.parent / "lib")
    + ":/nix/store/92iwsqgbgppkr2lqd5vd6fyff7vwvwg4-glew-2.3.1/lib"
)
Finalize("_ZN6Slic3r14GCodeProcessor13UsedFilaments14process_cachesEPS0_", internal=True)
if MODE == "slice":
    gdb.execute("set args --slice 0 --outputdir " + str(ROOT) + " " + str(SOURCE))
else:
    gdb.execute("set args --process-gcode " + str(SOURCE)
                + " --dump-color-change " + str(ROOT / "metadata.json")
                + " --dump-moves " + str(ROOT / "moves.csv"))
gdb.execute("run")
assert int(gdb.parse_and_eval("$_exitcode")) == 0
assert len(snapshots) == 1
assert len(printer_dialects) == 1
rows = snapshots[0]
gcode = (ROOT / "plate_1.gcode").read_bytes() if MODE == "slice" else SOURCE.read_bytes()
lines = gcode.decode().splitlines()


def footer(unit):
    prefix = "; filament used [" + unit + "] = "
    values = [line.removeprefix(prefix) for line in lines if line.startswith(prefix)]
    assert len(values) == 1 and "," not in values[0]
    return values[0]


meters = sum(row["meters"] for row in rows)
grams = sum(row["grams"] for row in rows)
# Footer is a rounded observable; the role-map assertions retain all f64 bits.
if MODE == "slice":
    assert f"{meters * 1000:.2f}" == footer("mm")
    assert f"{grams:.2f}" == footer("g")
(ROOT / "role-map.json").write_text(json.dumps({
    "binary_sha256": SHA,
    "input_sha256": hashlib.sha256(SOURCE.read_bytes()).hexdigest(),
    "gcode_sha256": hashlib.sha256(gcode).hexdigest(),
    "bbl_printer": printer_dialects[0],
    "roles": rows,
    "footer_check": {
        "meters_sum": meters, "grams_sum": grams,
        "mm": footer("mm"), "g": footer("g"), "passed": True,
        "scope": "same Native slice run, single filament, no flush",
    } if MODE == "slice" else None,
}, indent=2) + "\n")
