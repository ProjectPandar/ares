const encoder = new TextEncoder();

export const SHA = Object.freeze({
  fixture: "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
  ksrM: "91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19",
  ksrN: "42e0053bffb3093a44597abd0a2b4e8b8c8c11d6f07003cb894399ad7dce3c6e",
  parserN: "4a484de8c60e948744a73bae9d123103968de9fddda187ea6829982fbcd66f1c",
});

const flow = (fields, bridge, mm3) => Object.freeze([fields, bridge, mm3]);
export const FLOW = Object.freeze({
  INITIAL_LAYER: flow([0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd], false, "3fb76708c0000000"),
  INNER_ABSOLUTE_NOZZLE_04: flow([0x3ee66666, 0x3e4ccccd, 0x3ed06cbe, 0x3ecccccd], false, "3fb4d7aca0000000"),
  OUTER_ABSOLUTE_NOZZLE_04: flow([0x3ed70a3d, 0x3e4ccccd, 0x3ec11094, 0x3ecccccd], false, "3fb34e7540000000"),
  OVERHANG_FLOW_100_NOZZLE_04: flow([0x3ecccccd, 0x3e4ccccd, 0x3eb6d324, 0x3ecccccd], false, "3fb2485080000000"),
  INNER_ABSOLUTE_NOZZLE_06: flow([0x3ee66666, 0x3e4ccccd, 0x3ed06cbe, 0x3f19999a], false, "3fb4d7aca0000000"),
  OUTER_ABSOLUTE_NOZZLE_06: flow([0x3ed70a3d, 0x3e4ccccd, 0x3ec11094, 0x3f19999a], false, "3fb34e7540000000"),
  OVERHANG_FLOW_100_NOZZLE_06: flow([0x3f19999a, 0x3e4ccccd, 0x3f0e9cc6, 0x3f19999a], false, "3fbc85c120000000"),
  INNER_PERCENT_110_NOZZLE_06: flow([0x3f28f5c3, 0x3e4ccccd, 0x3f1df8ef, 0x3f19999a], false, "3fbf982fc0000000"),
  OUTER_PERCENT_125_NOZZLE_06: flow([0x3f400000, 0x3e4ccccd, 0x3f35032c, 0x3f19999a], false, "3fc219eac0000000"),
  OVERHANG_WIDTH_80_NOZZLE_06: flow([0x3ef5c290, 0x3e4ccccd, 0x3edfc8e8, 0x3f19999a], false, "3fb660e400000000"),
  SOLID_PERCENT_100_NOZZLE_06: flow([0x3f19999a, 0x3e4ccccd, 0x3f0e9cc6, 0x3f19999a], false, "3fbc85c120000000"),
  INNER_PERCENT_110_NOZZLE_04: flow([0x3ee147ae, 0x3e4ccccd, 0x3ecb4e06, 0x3ecccccd], false, "3fb4549a20000000"),
  OVERHANG_WIDTH_80_NOZZLE_04: flow([0x3ea3d70a, 0x3e4ccccd, 0x3e8ddd62, 0x3ecccccd], false, "3fac5f79e0000000"),
  OBJECT_WIDTH_052_FALLBACK: flow([0x3f051eb8, 0x3e4ccccd, 0x3ef443c8, 0x3ecccccd], false, "3fb86d2da0000000"),
  OVERHANG_FLOW_140: flow([0x3ed59710, 0x3e8f5c2a, 0x3eb6d324, 0x3ecccccd], false, "3fb99870c0000000"),
  OVERHANG_FLOW_080: flow([0x3ea83c2c, 0x3e4ccccd, 0x3e924284, 0x3ecccccd], false, "3fad4080c0000000"),
  OVERHANG_FLOW_020: flow([0x3d8a1779, 0x3d8a1779, 0x3eb6d324, 0x3ecccccd], false, "3f6d4080a0000000"),
  OVERHANG_AUTO_WIDTH_FLOW_080: flow([0x3ebcb70d, 0x3e4ccccd, 0x3ea6bd64, 0x3ecccccd], false, "3fb0ac8a00000000"),
  OVERHANG_WIDTH_120_FLOW_144_NONTHICK: flow([0x3eff6ddb, 0x3e9374bc, 0x3edfc8e8, 0x3ecccccd], false, "3fc01ccd20000000"),
  OVERHANG_AUTO_WIDTH_FLOW_064_NONTHICK: flow([0x3e9b5df8, 0x3e4ccccd, 0x3e856450, 0x3ecccccd], false, "3faaada980000000"),
  OVERHANG_WIDTH_120_FLOW_144_THICK: flow([0x3f1374bd, 0x3f1374bd, 0x3f20418a, 0x3ecccccd], true, "3fd0ad4840000000"),
  OVERHANG_AUTO_WIDTH_FLOW_064_THICK: flow([0x3ea3d70b, 0x3ea3d70b, 0x3ebd70a5, 0x3ecccccd], true, "3fb496b7e0000000"),
});
export const KSR_SAMPLES = Object.freeze({
  0: Object.freeze({
    geometry: [6, 0, 6, 6], height: "3fc999999999999a", sliceZ: "3fb999999999999a",
    flows: [FLOW.INITIAL_LAYER, FLOW.INITIAL_LAYER, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.INITIAL_LAYER],
  }),
  1: Object.freeze({
    geometry: [6, 6, 6, 6], height: "3fc999999999999a", sliceZ: "3fd3333333333334",
    flows: [FLOW.INNER_ABSOLUTE_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04],
  }),
  229: Object.freeze({
    geometry: [1, 1, 1, 1], height: "3fc9999999999a00", sliceZ: "4046f33333333343",
    flows: [FLOW.INNER_ABSOLUTE_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04],
  }),
  459: Object.freeze({
    geometry: [9, 9, 0, 0], height: "3fc9999999999a00", sliceZ: "4056f999999999d0",
    flows: [FLOW.INNER_ABSOLUTE_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04],
  }),
});

function put64(bytes, value) {
  let remaining = BigInt.asUintN(64, BigInt(value));
  for (let index = 0; index < 8; index += 1) {
    bytes.push(Number(remaining & 0xffn));
    remaining >>= 8n;
  }
}
function put32(bytes, value) {
  for (let index = 0; index < 4; index += 1) bytes.push((value >>> (8 * index)) & 0xff);
}

const POINTS = [[0n, 0n], [10n, 0n], [10n, 10n], [0n, 10n]];
const putList = (bytes, values, write) => {
  put64(bytes, values.length);
  for (const value of values) write(bytes, value);
};
const putPolygon = (bytes, points) => putList(bytes, points, (output, [x, y]) => {
  put64(output, x); put64(output, y);
});
const putExPolygon = (bytes) => { putPolygon(bytes, POINTS); put64(bytes, 0); };
const putSurfaces = (bytes, mark) => {
  put64(bytes, 1);
  if (mark) mark.surfaceKind = bytes.length;
  bytes.push(4);
  putExPolygon(bytes);
};
function parserPredecessor() {
  const bytes = Array.from(encoder.encode("ARES22M\0"));
  put64(bytes, 1);
  for (const value of [0, 0, 1, 0, 1, 0, 1, 0]) put64(bytes, value);
  put64(bytes, 1);
  bytes.push(4);
  putExPolygon(bytes);
  put64(bytes, 1);
  putExPolygon(bytes);
  return Uint8Array.from(bytes);
}
function putFlow(bytes, value, mark) {
  for (const field of value[0]) put32(bytes, field);
  if (mark) mark.flowBoolean = bytes.length;
  bytes.push(Number(value[1]));
  put64(bytes, `0x${value[2]}`);
}

export function parserKat() {
  const predecessor = parserPredecessor();
  const bytes = Array.from(encoder.encode("ARES22N\0"));
  const offsets = { predecessorLength: bytes.length };
  put64(bytes, predecessor.length);
  offsets.predecessorMagic = bytes.length;
  offsets.predecessorObjectCount = bytes.length + 8;
  bytes.push(...predecessor);
  offsets.objectCount = bytes.length;
  put64(bytes, 1);
  for (const value of [0, 0, 1, 1]) put64(bytes, value);
  offsets.slotPresence = bytes.length;
  bytes.push(1);
  for (const value of [0, 0, 0, 0, 0, 1, 0, 0, 0]) put64(bytes, value);
  bytes.push(0, 0, 0);
  putSurfaces(bytes, offsets);
  put64(bytes, "0x3fc999999999999a");
  put64(bytes, "0x3fb999999999999a");
  putFlow(bytes, FLOW.INITIAL_LAYER, offsets);
  for (const value of [FLOW.OUTER_ABSOLUTE_NOZZLE_04, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04]) putFlow(bytes, value);
  offsets.spiralBoolean = bytes.length;
  bytes.push(0);
  put64(bytes, 0);
  offsets.dispatch = bytes.length;
  bytes.push(0);
  return {
    bytes: Uint8Array.from(bytes),
    offsets,
    expected: Object.freeze({
      predecessorLength: predecessor.length,
      objectCount: 1,
      record: Object.freeze({
        identity: [0, 0, 0, 0, 0], compatible: [0], current: [0, 0],
        lower: null, upper: null, upperSame: null, geometry: [1, 0, 0, 0],
        height: "3fc999999999999a", sliceZ: "3fb999999999999a",
        flows: [FLOW.INITIAL_LAYER, FLOW.OUTER_ABSOLUTE_NOZZLE_04, FLOW.OVERHANG_FLOW_100_NOZZLE_04, FLOW.OUTER_ABSOLUTE_NOZZLE_04],
        spiral: false, rotation: "0000000000000000", dispatch: 0,
      }),
    }),
  };
}

export const PROCESS = "Metadata/project_settings.config";
export const MODEL = "Metadata/model_settings.config";
export const ROOT_PATH = "3D/3dmodel.model";
const LEAF_PATH = "3D/Objects/task22n_box.model";
const ROOT = [
  `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" `,
  `xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p">`,
  `<resources><object id="2" type="model"><components>`,
  `<component p:path="/${LEAF_PATH}" objectid="1"/></components></object></resources>`,
  `<build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" `,
  `auto_drop="1"/></build></model>`,
].join("");
const RELATIONSHIPS = [
  `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">`,
  `<Relationship Target="/${LEAF_PATH}" Id="box" `,
  `Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>`,
].join("");
const SETTINGS = [
  `<config><object id="2"><part id="1" subtype="normal_part"/></object><plate>`,
  `<metadata key="plater_id" value="1"/><model_instance>`,
  `<metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/>`,
  `<metadata key="identify_id" value="22001"/></model_instance></plate><assemble>`,
  `<assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" `,
  `offset="0 0 0"/></assemble></config>`,
].join("");
const LEAF = [
  `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">`,
  `<resources><object id="1" type="model"><mesh><vertices>`,
  `<vertex x="0" y="0" z="0"/><vertex x="8" y="0" z="0"/>`,
  `<vertex x="8" y="4" z="0"/><vertex x="4.6" y="4" z="0"/>`,
  `<vertex x="4.6" y="9" z="0"/><vertex x="3.4" y="9" z="0"/>`,
  `<vertex x="3.4" y="4" z="0"/><vertex x="0" y="4" z="0"/>`,
  `<vertex x="0" y="0" z="0.4"/><vertex x="8" y="0" z="0.4"/>`,
  `<vertex x="8" y="4" z="0.4"/><vertex x="4.6" y="4" z="0.4"/>`,
  `<vertex x="4.6" y="9" z="0.4"/><vertex x="3.4" y="9" z="0.4"/>`,
  `<vertex x="3.4" y="4" z="0.4"/><vertex x="0" y="4" z="0.4"/>`,
  `</vertices><triangles>`,
  `<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/>`,
  `<triangle v1="0" v2="6" v3="3"/><triangle v1="0" v2="7" v3="6"/>`,
  `<triangle v1="3" v2="5" v3="4"/><triangle v1="3" v2="6" v3="5"/>`,
  `<triangle v1="8" v2="9" v3="10"/><triangle v1="8" v2="10" v3="11"/>`,
  `<triangle v1="8" v2="11" v3="14"/><triangle v1="8" v2="14" v3="15"/>`,
  `<triangle v1="11" v2="12" v3="13"/><triangle v1="11" v2="13" v3="14"/>`,
  `<triangle v1="0" v2="1" v3="9"/><triangle v1="0" v2="9" v3="8"/>`,
  `<triangle v1="1" v2="2" v3="10"/><triangle v1="1" v2="10" v3="9"/>`,
  `<triangle v1="2" v2="3" v3="11"/><triangle v1="2" v2="11" v3="10"/>`,
  `<triangle v1="3" v2="4" v3="12"/><triangle v1="3" v2="12" v3="11"/>`,
  `<triangle v1="4" v2="5" v3="13"/><triangle v1="4" v2="13" v3="12"/>`,
  `<triangle v1="5" v2="6" v3="14"/><triangle v1="5" v2="14" v3="13"/>`,
  `<triangle v1="6" v2="7" v3="15"/><triangle v1="6" v2="15" v3="14"/>`,
  `<triangle v1="7" v2="0" v3="8"/><triangle v1="7" v2="8" v3="15"/>`,
  `</triangles></mesh></object></resources><build/></model>`,
].join("");
const SECOND_VERTICES = [
  `<vertex x="12" y="0" z="0"/><vertex x="14" y="0" z="0"/>`,
  `<vertex x="14" y="2" z="0"/><vertex x="12" y="2" z="0"/>`,
  `<vertex x="12" y="0" z="0.6"/><vertex x="14" y="0" z="0.6"/>`,
  `<vertex x="14" y="2" z="0.6"/><vertex x="12" y="2" z="0.6"/>`,
].join("");
const SECOND_TRIANGLES = [
  `<triangle v1="16" v2="18" v3="17"/><triangle v1="16" v2="19" v3="18"/>`,
  `<triangle v1="20" v2="21" v3="22"/><triangle v1="20" v2="22" v3="23"/>`,
  `<triangle v1="16" v2="17" v3="21"/><triangle v1="16" v2="21" v3="20"/>`,
  `<triangle v1="17" v2="18" v3="22"/><triangle v1="17" v2="22" v3="21"/>`,
  `<triangle v1="18" v2="19" v3="23"/><triangle v1="18" v2="23" v3="22"/>`,
  `<triangle v1="19" v2="16" v3="20"/><triangle v1="19" v2="20" v3="23"/>`,
].join("");

const quoted = (key, value) => `"${key}": "${value}"`;
const listed = (key, first, second) =>
  `"${key}": [\r\n\t\t"${first}",\r\n\t\t"${second}"\r\n\t]`;
const edit = (path, from, to) => [path, from, to];
const process = (key, from, to) => edit(PROCESS, quoted(key, from), quoted(key, to));
const change = (layers, roles, before, after) => ({ layers, roles, before, after });
const TWO_LAYERS = [0, 1];
const FIRST_LAYER = [0];
const INNER_ROLE = [0], OUTER_ROLE = [1], OVERHANG_ROLE = [2], SOLID_ROLE = [3];
const OUTER_AND_SOLID_ROLES = [1, 3];
const NON_OVERHANG_ROLES = [0, 1, 3];
const INITIAL_ZERO = process("initial_layer_line_width", "0.5", "0");
const NOZZLES_46 = edit(PROCESS, listed("nozzle_diameter", "0.4", "0.4"),
  listed("nozzle_diameter", "0.4", "0.6"));
const OUTER_TWO = process("outer_wall_filament_id", "0", "2");
const INNER_TWO = process("inner_wall_filament_id", "0", "2");
const SOLID_TWO = process("internal_solid_filament_id", "0", "2");
const widthZeroes = [
  process("outer_wall_line_width", "0.42", "0"),
  process("inner_wall_line_width", "0.45", "0"),
  process("internal_solid_infill_line_width", "0.42", "0"),
];
const flowPair = (name, setup, delta, changes = []) =>
  ({ name, kind: "flow", layers: 2, setup, delta, changes });
const selector = (name, key, changes) => flowPair(name,
  [INITIAL_ZERO, NOZZLES_46, process(key, "0", "1")], process(key, "1", "2"), changes);

export function flowPairs() {
  return [
    flowPair("initial", [], INITIAL_ZERO, [
      change(FIRST_LAYER, INNER_ROLE, FLOW.INITIAL_LAYER, FLOW.INNER_ABSOLUTE_NOZZLE_04),
      change(FIRST_LAYER, OUTER_AND_SOLID_ROLES,
        FLOW.INITIAL_LAYER, FLOW.OUTER_ABSOLUTE_NOZZLE_04),
    ]),
    flowPair("outer-percent", [INITIAL_ZERO, NOZZLES_46, OUTER_TWO],
      process("outer_wall_line_width", "0.42", "125%"),
      [change(TWO_LAYERS, OUTER_ROLE, FLOW.OUTER_ABSOLUTE_NOZZLE_06,
        FLOW.OUTER_PERCENT_125_NOZZLE_06)]),
    flowPair("inner-percent", [INITIAL_ZERO, NOZZLES_46, INNER_TWO],
      process("inner_wall_line_width", "0.45", "110%"),
      [change(TWO_LAYERS, INNER_ROLE, FLOW.INNER_ABSOLUTE_NOZZLE_06,
        FLOW.INNER_PERCENT_110_NOZZLE_06)]),
    flowPair("solid-percent", [INITIAL_ZERO, NOZZLES_46, SOLID_TWO],
      process("internal_solid_infill_line_width", "0.42", "100%"),
      [change(TWO_LAYERS, SOLID_ROLE, FLOW.OUTER_ABSOLUTE_NOZZLE_06,
        FLOW.SOLID_PERCENT_100_NOZZLE_06)]),
    flowPair("object-fallback", [INITIAL_ZERO, ...widthZeroes],
      process("line_width", "0.42", "0.52"),
      [change(TWO_LAYERS, NON_OVERHANG_ROLES, FLOW.OUTER_ABSOLUTE_NOZZLE_04,
        FLOW.OBJECT_WIDTH_052_FALLBACK)]),
    selector("outer-selector", "outer_wall_filament_id",
      [change(TWO_LAYERS, OUTER_ROLE, FLOW.OUTER_ABSOLUTE_NOZZLE_04,
        FLOW.OUTER_ABSOLUTE_NOZZLE_06)]),
    selector("inner-selector", "inner_wall_filament_id", [
      change(TWO_LAYERS, INNER_ROLE, FLOW.INNER_ABSOLUTE_NOZZLE_04,
        FLOW.INNER_ABSOLUTE_NOZZLE_06),
      change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_FLOW_100_NOZZLE_04,
        FLOW.OVERHANG_FLOW_100_NOZZLE_06),
    ]),
    selector("solid-selector", "internal_solid_filament_id",
      [change(TWO_LAYERS, SOLID_ROLE, FLOW.OUTER_ABSOLUTE_NOZZLE_04,
        FLOW.OUTER_ABSOLUTE_NOZZLE_06)]),
    flowPair("raw-zero-one", [], process("outer_wall_filament_id", "0", "1")),
    flowPair("scoped-fallback", [NOZZLES_46, edit(MODEL,
      `<object id="2"><part id="1" subtype="normal_part"/>`,
      [`<object id="2"><metadata key="extruder" value="2"/>`,
        `<part id="1" subtype="normal_part">`,
        `<metadata key="outer_wall_filament_id" value="0"/></part>`].join(""))],
    edit(MODEL, `<metadata key="outer_wall_filament_id" value="0"/>`,
      `<metadata key="outer_wall_filament_id" value="2"/>`)),
    flowPair("nozzle-list", [INITIAL_ZERO, OUTER_TWO, INNER_TWO, SOLID_TWO,
      process("outer_wall_line_width", "0.42", "125%"),
      process("inner_wall_line_width", "0.45", "110%"),
      process("internal_solid_infill_line_width", "0.42", "100%"),
      process("bridge_line_width", "100%", "80%")], NOZZLES_46, [
      change(TWO_LAYERS, INNER_ROLE, FLOW.INNER_PERCENT_110_NOZZLE_04,
        FLOW.INNER_PERCENT_110_NOZZLE_06),
      change(TWO_LAYERS, OUTER_ROLE, FLOW.INITIAL_LAYER, FLOW.OUTER_PERCENT_125_NOZZLE_06),
      change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_WIDTH_80_NOZZLE_04,
        FLOW.OVERHANG_WIDTH_80_NOZZLE_06),
      change(TWO_LAYERS, SOLID_ROLE, FLOW.OVERHANG_FLOW_100_NOZZLE_04,
        FLOW.SOLID_PERCENT_100_NOZZLE_06),
    ]),
    flowPair("anti-map", [INITIAL_ZERO, NOZZLES_46,
      process("outer_wall_filament_id", "0", "1"),
      process("inner_wall_filament_id", "0", "1"),
      process("internal_solid_filament_id", "0", "1"),
      edit(PROCESS, listed("filament_map", "1", "1"), listed("filament_map", "1", "2"))],
    edit(PROCESS, listed("filament_map", "1", "2"), listed("filament_map", "2", "1"))),
    flowPair("bridge-auto", [INITIAL_ZERO, process("bridge_flow", "1", "0.8")],
      process("bridge_line_width", "100%", "0"),
      [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_FLOW_080,
        FLOW.OVERHANG_AUTO_WIDTH_FLOW_080)]),
    flowPair("bridge-grow", [INITIAL_ZERO], process("bridge_flow", "1", "1.4"),
      [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_FLOW_100_NOZZLE_04,
        FLOW.OVERHANG_FLOW_140)]),
    flowPair("bridge-shrink", [INITIAL_ZERO], process("bridge_flow", "1", "0.8"),
      [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_FLOW_100_NOZZLE_04,
        FLOW.OVERHANG_FLOW_080)]),
    flowPair("bridge-round", [INITIAL_ZERO], process("bridge_flow", "1", "0.2"),
      [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_FLOW_100_NOZZLE_04,
        FLOW.OVERHANG_FLOW_020)]),
    flowPair("bridge-epsilon", [INITIAL_ZERO], process("bridge_flow", "1", "1.0005")),
    flowPair("thick-configured", [INITIAL_ZERO,
      process("bridge_line_width", "100%", "120%"), process("bridge_flow", "1", "1.44")],
    process("thick_bridges", "0", "1"),
    [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_WIDTH_120_FLOW_144_NONTHICK,
      FLOW.OVERHANG_WIDTH_120_FLOW_144_THICK)]),
    flowPair("thick-auto", [INITIAL_ZERO,
      process("bridge_line_width", "100%", "0"), process("bridge_flow", "1", "0.64")],
    process("thick_bridges", "0", "1"),
    [change(TWO_LAYERS, OVERHANG_ROLE, FLOW.OVERHANG_AUTO_WIDTH_FLOW_064_NONTHICK,
      FLOW.OVERHANG_AUTO_WIDTH_FLOW_064_THICK)]),
  ];
}

const ARACHNE = process("wall_generator", "classic", "arachne");
const ALIGN = process("align_infill_direction_to_model", "0", "1");
const IDENTITY = `transform="1 0 0 0 1 0 0 0 1 0 0 0"`;
const context = (spiral, dispatch, rotation = "0000000000000000") =>
  [spiral, dispatch, rotation];
const triple = (value) => [value, value, value];
const contextPair = (name, setup, delta, contexts, m = null) =>
  ({ name, kind: "context", layers: 3, setup, delta, contexts, m });

export function contextPairs() {
  const classic = context(false, 0);
  return [
    contextPair("alignment", [edit(ROOT_PATH, IDENTITY,
      `transform="0 1 0 -1 0 0 0 0 1 0 0 0"`)], ALIGN,
    [triple(classic), triple(context(false, 0, "3ff921fb54442d18"))]),
    contextPair("signed-zero", [ALIGN], edit(ROOT_PATH, IDENTITY,
      `transform="1 -0 0 0 1 0 0 0 1 0 0 0"`),
    [triple(classic), triple(context(false, 0, "8000000000000000"))]),
    contextPair("generator", [], ARACHNE,
      [triple(classic), triple(context(false, 1))]),
  ];
}

export function replaceUnique(text, before, after) {
  const first = text.indexOf(before);
  if (first < 0 || text.indexOf(before, first + before.length) >= 0) {
    throw new Error(`expected exactly one ${before}`);
  }
  return text.slice(0, first) + after + text.slice(first + before.length);
}

export function syntheticEntries(projectSettings, threeLayers) {
  let leaf = LEAF;
  if (threeLayers) {
    leaf = leaf.replaceAll('z="0.4"', 'z="0.6"')
      .replace("</vertices>", `${SECOND_VERTICES}</vertices>`)
      .replace("</triangles>", `${SECOND_TRIANGLES}</triangles>`);
  }
  const process = replaceUnique(
    projectSettings,
    `"elefant_foot_compensation": "0.15"`,
    `"elefant_foot_compensation": "0"`,
  );
  return [
    [ROOT_PATH, ROOT],
    ["3D/_rels/3dmodel.model.rels", RELATIONSHIPS],
    [LEAF_PATH, leaf],
    [MODEL, SETTINGS],
    [PROCESS, process],
  ];
}
