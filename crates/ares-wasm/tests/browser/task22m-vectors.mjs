const encoder = new TextEncoder();

export const SHA = Object.freeze({
  fixture: "698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9",
  ksrL: "7a71db2912970141adc436679621c25888c412e2010c44eccf1b49d7e8048b07",
  ksrM: "91f6943a67fb7b42acbf6d4fbf9c98bc4bb91815df888ff5a99184bf53728d19",
  parserM: "74b339adcebb586f48a8b390e097b61b399d644cb3c1e7bb926b89d2fbfd0f07",
  smallL: "70c9c246700b068e1085a2c719243fd94839bb169c3a062b06b42fd640147b2a",
  smallMDisabled: "868e82681ed9712461329ea54952c63ec05be1c6b0229f2622c7fed493adc55a",
  smallMEnabled: "bb0fcb21a733f65462c5a669c6f46895ab16c049a342acafbb98bb376e05560e",
  archiveDisabled: "eb6d0edbf190f9449ad089da587806f4b0f4a1d5e2fd18b05ce91830113089d0",
  archiveEnabled: "7f22881be0eb1c4aa6f1997639fc15babc43413ee524ce41b664435e1c69834b",
  semanticDisabled: "0dcd709d31754d2340c5b82df4871fcd69bacf12800df540c02e513512f26fce",
  semanticEnabled: "df965ba633362a23b19ad9f9fef62b0397a5262b61e3e8da349d9f588237c073",
});

export const RECORDS = Object.freeze({
  sidecar: [
    [11_680, "bbc99a45cc9a566fefdbc4a7fa1ae80865858126f2ba0a9b9ee9c412f8414581"],
    [24_216, "47486ac767ceea0b822566a750abc913c326141ca91eef5b27cfc1b37d26de4d"],
    [23_512, "ec3c90e0e8d276b9995169285b5b5a939e60bbd7283e46d0fa2c299bd8756816"],
    [736, "fd1b4912b9472d854d664769d1d0e5c5ec49e9bb9efd67e43c5707bca9189d0a"],
  ],
  retained: [
    [27_502, "15481fd8a31da16d14d52e2b12f72de267470021331aeb3d390849e14f63b151"],
    [48_456, "5e3677fee988aa21c0949a827941ea2781ca5d355b0be11336a8a302206f13cd"],
    [47_048, "3b91765a18eeb9f9de3223e8155c4eecf693b0d2262421240ff0abcdf67e656c"],
    [1_489, "226ee0e0b2b3a823555836f1cbc8f3deaa256c48f40d3b7e5803d156d5a5272c"],
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

function putRetainedLayer(bytes, layer, withLslices) {
  put64(bytes, layer.index);
  putList(bytes, layer.regions, (output, region) => {
    put64(output, region.id);
    putList(output, region.surfaces, (surfaceBytes, surface) => {
      surfaceBytes.push(surface.type);
      putExPolygon(surfaceBytes, surface.expolygon);
    });
  });
  if (withLslices) putList(bytes, layer.lslices, putExPolygon);
}

function encodeCheckpoint(magic, objects) {
  const bytes = Array.from(encoder.encode(magic));
  const withLslices = magic === "ARES22M\0";
  putList(bytes, objects, (output, object) => {
    put64(output, object.sourceObjectIndex);
    put64(output, object.transformIndex);
    put64(output, object.plannedLayerCount);
    putList(output, object.sidecars, (sidecarBytes, sidecar) => {
      put64(sidecarBytes, sidecar.occurrenceId);
      putList(sidecarBytes, sidecar.layers, putLayerGeometry);
    });
    putList(output, object.retainedLayers, (layerBytes, layer) =>
      putRetainedLayer(layerBytes, layer, withLslices));
  });
  return Uint8Array.from(bytes);
}

const expolygon = (contour, holes = []) => ({ contour, holes });
const internal = (contour, holes = []) => ({ type: 4, expolygon: expolygon(contour, holes) });
const wirePoints = (points) => points.map(([x, y]) => [BigInt(x), BigInt(y)]);
const astPoints = (points) => points.map(([x, y]) => [String(x), String(y)]);

function expectedAst(magic, byteLength, objects) {
  const exp = (value) => ({
    contour: astPoints(value.contour),
    holes: value.holes.map(astPoints),
  });
  const retained = (layer) => ({
    index: layer.index,
    regions: layer.regions.map((region) => ({
      id: region.id,
      surfaces: region.surfaces.map((surface) => ({
        type: surface.type, expolygon: exp(surface.expolygon),
      })),
    })),
    ...(magic === "ARES22M\0" ? { lslices: layer.lslices.map(exp) } : {}),
  });
  return {
    magic, byteLength, bytesRead: byteLength,
    objects: objects.map((object) => ({
      sourceObjectIndex: object.sourceObjectIndex,
      transformIndex: object.transformIndex,
      plannedLayerCount: object.plannedLayerCount,
      sidecars: object.sidecars.map((sidecar) => ({
        occurrenceId: sidecar.occurrenceId,
        layers: sidecar.layers.map((layer) => ({
          index: layer.index,
          expolygons: layer.expolygons.map(exp),
        })),
      })),
      retainedLayers: object.retainedLayers.map(retained),
    })),
  };
}

function vector(magic, objects, sha256) {
  const bytes = encodeCheckpoint(magic, objects);
  return { bytes, expected: expectedAst(magic, bytes.length, objects), sha256 };
}

const RAW = wirePoints([
  [4_000_000, -500_000], [600_000, -500_000], [600_000, 4_500_000],
  [-600_000, 4_500_000], [-600_000, -500_000], [-4_000_000, -500_000],
  [-4_000_000, -4_500_000], [4_000_000, -4_500_000],
]);
const ENABLED = wirePoints([
  [3_850_000, -650_000], [2_542_857, -650_000], [2_057_142, -649_904],
  [1_571_428, -648_459], [1_085_714, -640_528], [478_540, -621_460],
  [478_540, 3_500_000], [459_472, 4_000_000], [453_904, 4_351_013],
  [200_000, 4_350_096], [-200_000, 4_350_096], [-453_904, 4_351_013],
  [-459_472, 4_000_000], [-478_540, 3_500_000], [-478_540, -621_460],
  [-1_085_714, -640_528], [-1_571_428, -648_459], [-2_057_142, -649_904],
  [-2_542_857, -650_000], [-3_850_000, -650_000], [-3_850_000, -4_350_000],
  [3_850_000, -4_350_000],
]);

function smallObject(firstContour, withLslices) {
  const layer = (index, contour) => ({
    index,
    regions: [{ id: 0, surfaces: [internal(contour)] }],
    ...(withLslices ? { lslices: [expolygon(RAW)] } : {}),
  });
  return {
    sourceObjectIndex: 0,
    transformIndex: 0,
    plannedLayerCount: 2,
    sidecars: [{ occurrenceId: 1, layers: [
      { index: 0, expolygons: [expolygon(RAW)] },
      { index: 1, expolygons: [expolygon(RAW)] },
    ] }],
    retainedLayers: [layer(0, firstContour), layer(1, RAW)],
  };
}

export function smallKats() {
  return {
    input: vector("ARES22L\0", [smallObject(RAW, false)], SHA.smallL),
    disabled: vector("ARES22M\0", [smallObject(RAW, true)], SHA.smallMDisabled),
    enabled: vector("ARES22M\0", [smallObject(ENABLED, true)], SHA.smallMEnabled),
  };
}

const LEFT_HOLE = wirePoints([[10, 10], [10, 50], [50, 50], [50, 10]]);
const RIGHT_HOLE = wirePoints([[110, 10], [110, 50], [150, 50], [150, 10]]);
const LEFT_RAW = wirePoints([[0, 0], [60, 0], [60, 60], [0, 60]]);
const NESTED_RAW = wirePoints([[20, 20], [40, 20], [40, 40], [20, 40]]);
const RIGHT_RAW = wirePoints([[100, 0], [160, 0], [160, 60], [100, 60]]);
const LEFT_SURFACE = wirePoints([[60, 60], [0, 60], [0, 0], [60, 0]]);
const NESTED_SURFACE = wirePoints([[40, 40], [20, 40], [20, 20], [40, 20]]);
const RIGHT_SURFACE = wirePoints([[160, 60], [100, 60], [100, 0], [160, 0]]);

function parserMObject() {
  return {
    sourceObjectIndex: 18,
    transformIndex: 0,
    plannedLayerCount: 1,
    sidecars: [],
    retainedLayers: [{
      index: 0,
      regions: [{ id: 0, surfaces: [
        internal(LEFT_SURFACE, [LEFT_HOLE]),
        internal(NESTED_SURFACE),
        internal(RIGHT_SURFACE, [RIGHT_HOLE]),
      ] }],
      lslices: [
        expolygon(RIGHT_RAW, [RIGHT_HOLE]),
        expolygon(NESTED_RAW),
        expolygon(LEFT_RAW, [LEFT_HOLE]),
      ],
    }],
  };
}

export function parserKats() {
  return {
    l: smallKats().input,
    m: vector("ARES22M\0", [parserMObject()], SHA.parserM),
  };
}

const ROOT = `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22m_box.model" objectid="1"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>`;
const RELATIONSHIPS = `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22m_box.model" Id="box" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>`;
const SETTINGS = `<config><object id="2"><part id="1" subtype="normal_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>`;
const LEAF = `<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="1" type="model"><mesh><vertices>
<vertex x="0" y="0" z="0"/><vertex x="8" y="0" z="0"/><vertex x="8" y="4" z="0"/><vertex x="4.6" y="4" z="0"/><vertex x="4.6" y="9" z="0"/><vertex x="3.4" y="9" z="0"/><vertex x="3.4" y="4" z="0"/><vertex x="0" y="4" z="0"/>
<vertex x="0" y="0" z="0.4"/><vertex x="8" y="0" z="0.4"/><vertex x="8" y="4" z="0.4"/><vertex x="4.6" y="4" z="0.4"/><vertex x="4.6" y="9" z="0.4"/><vertex x="3.4" y="9" z="0.4"/><vertex x="3.4" y="4" z="0.4"/><vertex x="0" y="4" z="0.4"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="0" v2="6" v3="3"/><triangle v1="0" v2="7" v3="6"/><triangle v1="3" v2="5" v3="4"/><triangle v1="3" v2="6" v3="5"/>
<triangle v1="8" v2="9" v3="10"/><triangle v1="8" v2="10" v3="11"/><triangle v1="8" v2="11" v3="14"/><triangle v1="8" v2="14" v3="15"/><triangle v1="11" v2="12" v3="13"/><triangle v1="11" v2="13" v3="14"/>
<triangle v1="0" v2="1" v3="9"/><triangle v1="0" v2="9" v3="8"/><triangle v1="1" v2="2" v3="10"/><triangle v1="1" v2="10" v3="9"/><triangle v1="2" v2="3" v3="11"/><triangle v1="2" v2="11" v3="10"/><triangle v1="3" v2="4" v3="12"/><triangle v1="3" v2="12" v3="11"/>
<triangle v1="4" v2="5" v3="13"/><triangle v1="4" v2="13" v3="12"/><triangle v1="5" v2="6" v3="14"/><triangle v1="5" v2="14" v3="13"/><triangle v1="6" v2="7" v3="15"/><triangle v1="6" v2="15" v3="14"/><triangle v1="7" v2="0" v3="8"/><triangle v1="7" v2="8" v3="15"/>
</triangles></mesh></object></resources><build/></model>`;

export const SMALL_OPTION_REPLACEMENTS = Object.freeze({
  enabled: `"elefant_foot_compensation": "0.15"`,
  disabled: `"elefant_foot_compensation": "0"`,
});

function replaceUnique(text, before, after) {
  const first = text.indexOf(before);
  if (first < 0 || text.indexOf(before, first + before.length) >= 0) {
    throw new Error(`expected exactly one ${before}`);
  }
  return text.slice(0, first) + after + text.slice(first + before.length);
}

export function smallArchiveReplacements(enabled, projectSettings) {
  const { enabled: before, disabled: after } = SMALL_OPTION_REPLACEMENTS;
  if (projectSettings.split(before).length !== 2) {
    throw new Error(`expected exactly one ${before}`);
  }
  const process = enabled ? projectSettings : replaceUnique(projectSettings, before, after);
  return [
    ["3D/3dmodel.model", ROOT],
    ["3D/_rels/3dmodel.model.rels", RELATIONSHIPS],
    ["3D/Objects/task22m_box.model", LEAF],
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
