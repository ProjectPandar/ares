const encoder = new TextEncoder();

export const SHA = Object.freeze({
  fixture: "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
  ksrK: "c101e0f9ff863c7abe72cd1cb792fcd8e0074d8d6d2e77d3bb56c32eedba13be",
  ksrL: "7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07",
  katJ: "940f01934309cf1a23afe67e7d8365ced3e9f8296f8ee4db73261aac74e71a6a",
  katK: "a49fcd311d79d216d874c585ae107f33a178fd47e99d3f862475295d0e237751",
  stepK: "c6668cfbc56b20abe71606d59d2e28abf08ebb8b22f3ecebb3058d63ba05b44f",
  stepLDisabled: "0834c61cc48aece1afd52d060c5c2a58f7243124664ad0a7dd3f500d6735b790",
  stepLEnabled: "33038c51ffe6f41b0bdb8b921d6976f43b0c47f6f3be8ec3bee6cc5b9c7c2505",
  archiveDisabled: "c4c0ea05709a6fadd8b2d0d6d34dab1cad5420865c5993b58b9d8e91a8f73313",
  archiveEnabled: "130260c5c63846759aa66d25e68ff9bb07cf5aeec86ef7da9476c12761f3836d",
  semanticDisabled: "ade484830a6492b50c3233e51debf5eab1db7d3e3bbf81fa8cd72f10226ea9ef",
  semanticEnabled: "f61089d040d1edf002f1dedca66b433e4982e18b9ce69a6385aa42dbf4c780b9",
});

export const RECORDS = Object.freeze({
  sidecar: [
    [11_680, "bbc99a45cc9a566fefdbc4a7fa1ae80865858126f2ba0a9b9ee9c412f8414581"],
    [24_216, "47486ac767ceea0b822566a750abc913c326141ca91eef5b27cfc1b37d26de4d"],
    [23_512, "ec3c90e0e8d276b9995169285b5b5a939e60bbd7283e46d0fa2c299bd8756816"],
    [736, "fd1b4912b9472d854d664769d1d0e5c5ec49e9bb9efd67e43c5707bca9189d0a"],
  ],
  retained: [
    [11_702, "633fcb207ed0be4092a75c7ad6052fa68579c4ced58371afa8837cd99d65c21e"],
    [24_248, "486a43246ef4bc94b2119a4b5787662ff65162c416137caf5d131c1ea5d458ec"],
    [23_544, "59eaf433513f5c92203cbd58b10612fb7b3438c627666d6e7a5dae24711c86ea"],
    [761, "a19b98ff4513317e141d1dac1c7f978f60b50602210b7d1bd4afd94c9b4fe82d"],
  ],
});

function put64(bytes, value) {
  let remaining = BigInt.asUintN(64, BigInt(value));
  for (let index = 0; index < 8; index += 1) {
    bytes.push(Number(remaining & 0xffn));
    remaining >>= 8n;
  }
}

function putList(bytes, values, write) {
  put64(bytes, values.length);
  for (const value of values) write(bytes, value);
}

function putPolygon(bytes, points) {
  putList(bytes, points, (output, [x, y]) => {
    put64(output, x);
    put64(output, y);
  });
}

function putExPolygon(bytes, value) {
  putPolygon(bytes, value.contour);
  putList(bytes, value.holes, putPolygon);
}

function putLayerGeometry(bytes, layer) {
  put64(bytes, layer.index);
  putList(bytes, layer.expolygons, putExPolygon);
}

function putRetainedLayer(bytes, layer) {
  put64(bytes, layer.index);
  putList(bytes, layer.regions, (output, region) => {
    put64(output, region.id);
    putList(output, region.surfaces, (surfaceBytes, surface) => {
      surfaceBytes.push(surface.type);
      putExPolygon(surfaceBytes, surface.expolygon);
    });
  });
}

function encodeCheckpoint(magic, objects) {
  const bytes = Array.from(encoder.encode(magic));
  putList(bytes, objects, (output, object) => {
    put64(output, object.sourceObjectIndex);
    put64(output, object.transformIndex);
    put64(output, object.plannedLayerCount);
    putList(output, object.sidecars, (sidecarBytes, sidecar) => {
      put64(sidecarBytes, sidecar.occurrenceId);
      putList(sidecarBytes, sidecar.layers, putLayerGeometry);
    });
    putList(output, object.retainedLayers, putRetainedLayer);
  });
  return Uint8Array.from(bytes);
}

const expolygon = (contour, holes = []) => ({ contour, holes });
const internal = (contour) => ({ type: 4, expolygon: expolygon(contour) });
const wirePoint = ([x, y]) => [BigInt(x), BigInt(y)];
const astPoint = ([x, y]) => [String(x), String(y)];

function expectedAst(magic, byteLength, objects) {
  const polygon = (points) => points.map(astPoint);
  const exp = (value) => ({
    contour: polygon(value.contour),
    holes: value.holes.map(polygon),
  });
  return {
    magic,
    byteLength,
    bytesRead: byteLength,
    objects: objects.map((object) => ({
      ...object,
      sidecars: object.sidecars.map((sidecar) => ({
        ...sidecar,
        layers: sidecar.layers.map((layer) => ({
          ...layer,
          expolygons: layer.expolygons.map(exp),
        })),
      })),
      retainedLayers: object.retainedLayers.map((layer) => ({
        ...layer,
        regions: layer.regions.map((region) => ({
          ...region,
          surfaces: region.surfaces.map((surface) => ({
            type: surface.type,
            expolygon: exp(surface.expolygon),
          })),
        })),
      })),
    })),
  };
}

function vector(magic, objects, sha256) {
  const bytes = encodeCheckpoint(magic, objects);
  return { bytes, expected: expectedAst(magic, bytes.length, objects), sha256 };
}

const WIRE_OUTER = [
  [9_007_199_254_740_993n, -9_007_199_254_740_993n],
  [-40n, -40n], [-40n, 40n], [40n, 40n],
];
const WIRE_HOLE = [[-30n, -30n], [30n, -30n], [30n, 30n], [-30n, 30n]];

const parserObject = (retainedLayers) => ({
  sourceObjectIndex: 7,
  transformIndex: 9,
  plannedLayerCount: retainedLayers.length,
  sidecars: [{ occurrenceId: 3, layers: [
    { index: 0, expolygons: [expolygon(WIRE_OUTER, [WIRE_HOLE])] },
    { index: 1, expolygons: [] },
  ] }],
  retainedLayers,
});

const PARSER_LAYER_0 = { index: 0, regions: [
  { id: 0, surfaces: [internal(WIRE_OUTER)] },
  { id: 1, surfaces: [] },
] };
const PARSER_LAYER_1 = { index: 1, regions: [
  { id: 0, surfaces: [] }, { id: 1, surfaces: [] },
] };

export function parserKats() {
  return [
    vector("ARES22J\0", [parserObject([PARSER_LAYER_0, PARSER_LAYER_1])], SHA.katJ),
    vector("ARES22K\0", [parserObject([PARSER_LAYER_0])], SHA.katK),
  ];
}

const LOWER = [[1_000_000, 3_000_000], [-5_000_000, 3_000_000],
  [-5_000_000, -3_000_000], [1_000_000, -3_000_000]].map(wirePoint);
const UPPER = [[5_000_000, 3_000_000], [-1_000_000, 3_000_000],
  [-1_000_000, -3_000_000], [5_000_000, -3_000_000]].map(wirePoint);
const CHANGED_LOWER = [[1_000_000, -2_800_100], [4_800_100, -2_800_100],
  [4_800_100, 2_800_100], [1_000_000, 2_800_100], [1_000_000, 3_000_000],
  [-5_000_000, 3_000_000], [-5_000_000, -3_000_000], [1_000_000, -3_000_000]].map(wirePoint);

function steppedObject(lower) {
  return {
    sourceObjectIndex: 0,
    transformIndex: 0,
    plannedLayerCount: 2,
    sidecars: [{ occurrenceId: 1, layers: [
      { index: 0, expolygons: [expolygon(LOWER)] },
      { index: 1, expolygons: [expolygon(UPPER)] },
    ] }],
    retainedLayers: [
      { index: 0, regions: [{ id: 0, surfaces: [internal(lower)] }] },
      { index: 1, regions: [{ id: 0, surfaces: [internal(UPPER)] }] },
    ],
  };
}

export function steppedKats() {
  const original = [steppedObject(LOWER)];
  return {
    input: vector("ARES22K\0", original, SHA.stepK),
    disabled: vector("ARES22L\0", original, SHA.stepLDisabled),
    enabled: vector("ARES22L\0", [steppedObject(CHANGED_LOWER)], SHA.stepLEnabled),
  };
}

const ROOT = `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22l_step.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>`;
const RELATIONSHIPS = `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22l_step.model" Id="step" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>`;
const SETTINGS = `<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>`;
const LEAF = `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/><vertex x="6" y="0" z="0"/><vertex x="6" y="6" z="0"/><vertex x="0" y="6" z="0"/><vertex x="0" y="0" z="0.2"/><vertex x="6" y="0" z="0.2"/><vertex x="6" y="6" z="0.2"/><vertex x="0" y="6" z="0.2"/><vertex x="4" y="0" z="0.2"/><vertex x="10" y="0" z="0.2"/><vertex x="10" y="6" z="0.2"/><vertex x="4" y="6" z="0.2"/><vertex x="4" y="0" z="0.4"/><vertex x="10" y="0" z="0.4"/><vertex x="10" y="6" z="0.4"/><vertex x="4" y="6" z="0.4"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/><triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/><triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/><triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/><triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/><triangle v1="8" v2="10" v3="9"/><triangle v1="8" v2="11" v3="10"/><triangle v1="12" v2="13" v3="14"/><triangle v1="12" v2="14" v3="15"/><triangle v1="8" v2="9" v3="13"/><triangle v1="8" v2="13" v3="12"/><triangle v1="9" v2="10" v3="14"/><triangle v1="9" v2="14" v3="13"/><triangle v1="10" v2="11" v3="15"/><triangle v1="10" v2="15" v3="14"/><triangle v1="11" v2="8" v3="12"/><triangle v1="11" v2="12" v3="15"/></triangles></mesh></object></resources><build/></model>`;

export const STEPPED_OPTION_REPLACEMENTS = Object.freeze({
  angle: [`"make_overhang_printable_angle": "55"`, `"make_overhang_printable_angle": "45"`],
  disabled: [`"make_overhang_printable": "0"`, `"make_overhang_printable": "0"`],
  enabled: [`"make_overhang_printable": "0"`, `"make_overhang_printable": "1"`],
});

function replaceUnique(text, [before, after]) {
  const first = text.indexOf(before);
  if (first < 0 || text.indexOf(before, first + before.length) >= 0) {
    throw new Error(`expected exactly one ${before}`);
  }
  return text.slice(0, first) + after + text.slice(first + before.length);
}

export function steppedArchiveReplacements(enabled, projectSettings) {
  let process = replaceUnique(projectSettings, STEPPED_OPTION_REPLACEMENTS.angle);
  process = replaceUnique(
    process,
    enabled ? STEPPED_OPTION_REPLACEMENTS.enabled : STEPPED_OPTION_REPLACEMENTS.disabled,
  );
  return [
    ["3D/3dmodel.model", ROOT],
    ["3D/_rels/3dmodel.model.rels", RELATIONSHIPS],
    ["3D/Objects/task22l_step.model", LEAF],
    ["Metadata/model_settings.config", SETTINGS],
    ["Metadata/project_settings.config", process],
  ];
}

export function semanticBytes(entries) {
  const pairs = (Array.isArray(entries) ? entries : Object.entries(entries))
    .map(([name, bytes]) => [name, typeof bytes === "string" ? encoder.encode(bytes) : bytes]);
  pairs.sort(([left], [right]) => (left < right ? -1 : Number(left > right)));
  const parts = pairs.flatMap(([name, bytes]) => [encoder.encode(name), new Uint8Array(1), bytes]);
  const output = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) { output.set(part, offset); offset += part.length; }
  return output;
}
