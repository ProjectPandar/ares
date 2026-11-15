import assert from "node:assert/strict";
import { test } from "node:test";
import { PRINTER_PAGES } from "../app/src/sidebar/presetLayout.ts";

const basic = PRINTER_PAGES.find((page) => page.title === "Basic information");
const keys = (title) => basic.groups.find((group) => group.title === title).rows.map((row) => row.key);

test("printer rows stay with the Advanced receiver after scoped groups", () => {
  const advanced = keys("Advanced");
  assert.deepEqual(advanced.slice(advanced.indexOf("disable_m73")), [
    "disable_m73", "thumbnails", "use_relative_e_distances", "use_firmware_retraction",
    "time_cost", "build_plate_tilt_x", "build_plate_tilt_y",
  ]);
});

test("belt and machine frame groups retain their own rows and creation order", () => {
  const titles = basic.groups.map((group) => group.title);
  const start = titles.indexOf("Advanced");
  assert.deepEqual(titles.slice(start, start + 4), [
    "Advanced", "Belt printer", "Machine frame transforms", "Plugin Configuration",
  ]);
  assert.deepEqual(keys("Belt printer"), [
    "belt_printer", "belt_printer_infinite_y", "belt_slice_rotation_angle",
    "belt_slice_rotation", "belt_support_floor_offset",
  ]);
  assert.deepEqual(keys("Machine frame transforms"), [
    "gcode_remap_x", "gcode_remap_y", "gcode_remap_z",
    "belt_frame_tilt_decouple", "belt_frame_tilt_angle",
  ]);
  assert.deepEqual(keys("Plugin Configuration"), ["printer_plugin_config_overrides"]);
});
