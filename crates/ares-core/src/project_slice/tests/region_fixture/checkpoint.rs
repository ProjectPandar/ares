use sha2::{Digest, Sha256};

macro_rules! record {
    ($name:ident($($field:ident: $ty:ty),+ $(,)?)) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(in crate::project_slice::tests) struct $name {
            $(pub(in crate::project_slice::tests) $field: $ty),+
        }
    };
}
macro_rules! number {
    ($name:ident, $ty:ty, $bytes:expr) => {
        fn $name(&mut self) -> $ty {
            <$ty>::from_le_bytes(self.take::<$bytes>())
        }
    };
}
record!(ExPolygon(contour: Vec<(i64, i64)>, holes: Vec<Vec<(i64, i64)>>));
record!(IStream(objects: Vec<IObject>));
record!(IObject(source_object_index: u64, transform_index: u64, planned_layer_count: u64, volumes: Vec<IVolume>));
record!(IVolume(source_volume_index: u64, ordinal: u32, volume_type: u8, layers: Vec<ILayer>));
record!(ILayer(index: u64, mode: u8, expolygons: Vec<ExPolygon>));
record!(JStream(objects: Vec<JObject>));
record!(JObject(source_object_index: u64, transform_index: u64, planned_layer_count: u64, sidecars: Vec<Sidecar>, retained_layers: Vec<RetainedLayer>));
record!(Sidecar(occurrence_id: u64, layers: Vec<GeometryLayer>));
record!(GeometryLayer(index: u64, expolygons: Vec<ExPolygon>));
record!(RetainedLayer(index: u64, regions: Vec<Region>));
record!(Region(id: u64, surfaces: Vec<Surface>));
record!(Surface(kind: u8, expolygon: ExPolygon));

pub(in crate::project_slice::tests) struct ParsedJ {
    pub(in crate::project_slice::tests) stream: JStream,
    pub(in crate::project_slice::tests) sidecar_records: Vec<std::ops::Range<usize>>,
    pub(in crate::project_slice::tests) retained_records: Vec<std::ops::Range<usize>>,
}

pub(super) fn parse_i(bytes: &[u8], magic: &[u8; 8]) -> IStream {
    let mut reader = Reader::new(bytes, magic);
    let objects = reader.count_map(|reader| IObject {
        source_object_index: reader.u64(),
        transform_index: reader.u64(),
        planned_layer_count: reader.u64(),
        volumes: reader.count_map(|reader| IVolume {
            source_volume_index: reader.u64(),
            ordinal: reader.u32(),
            volume_type: reader.u8(),
            layers: reader.count_map(|reader| ILayer {
                index: reader.u64(),
                mode: reader.u8(),
                expolygons: reader.expolygons(),
            }),
        }),
    });
    reader.eof();
    IStream { objects }
}

pub(in crate::project_slice::tests) fn parse_j(bytes: &[u8]) -> ParsedJ {
    parse_post_regions(bytes, b"ARES22J\0")
}

pub(in crate::project_slice::tests) fn parse_k(bytes: &[u8]) -> ParsedJ {
    parse_post_regions(bytes, b"ARES22K\0")
}

pub(in crate::project_slice::tests) fn parse_l(bytes: &[u8]) -> ParsedJ {
    parse_post_regions(bytes, b"ARES22L\0")
}

fn parse_post_regions(bytes: &[u8], magic: &[u8; 8]) -> ParsedJ {
    let mut reader = Reader::new(bytes, magic);
    let mut sidecar_records = Vec::new();
    let mut retained_records = Vec::new();
    let objects = reader.count_map(|reader| {
        let source_object_index = reader.u64();
        let transform_index = reader.u64();
        let planned_layer_count = reader.u64();
        let sidecars = reader.count_map(|reader| Sidecar {
            occurrence_id: reader.u64(),
            layers: reader.count_map(|reader| {
                let start = reader.1;
                let layer = GeometryLayer {
                    index: reader.u64(),
                    expolygons: reader.expolygons(),
                };
                sidecar_records.push(start..reader.1);
                layer
            }),
        });
        let retained_layers = reader.count_map(|reader| {
            let start = reader.1;
            let layer = RetainedLayer {
                index: reader.u64(),
                regions: reader.count_map(|reader| Region {
                    id: reader.u64(),
                    surfaces: reader.count_map(|reader| Surface {
                        kind: reader.u8(),
                        expolygon: reader.expolygon(),
                    }),
                }),
            };
            retained_records.push(start..reader.1);
            layer
        });
        JObject {
            source_object_index,
            transform_index,
            planned_layer_count,
            sidecars,
            retained_layers,
        }
    });
    reader.eof();
    ParsedJ {
        stream: JStream { objects },
        sidecar_records,
        retained_records,
    }
}

pub(super) fn encode_j(stream: &JStream) -> Vec<u8> {
    encode_with_magic(stream, b"ARES22J\0")
}

pub(in crate::project_slice::tests) fn encode_with_magic(
    stream: &JStream,
    magic: &[u8; 8],
) -> Vec<u8> {
    let mut bytes = magic.to_vec();
    put_vec(&mut bytes, &stream.objects, |bytes, object| {
        put_u64(bytes, object.source_object_index);
        put_u64(bytes, object.transform_index);
        put_u64(bytes, object.planned_layer_count);
        put_vec(bytes, &object.sidecars, |bytes, sidecar| {
            put_u64(bytes, sidecar.occurrence_id);
            put_vec(bytes, &sidecar.layers, |bytes, layer| {
                put_u64(bytes, layer.index);
                put_expolygons(bytes, &layer.expolygons);
            });
        });
        put_vec(bytes, &object.retained_layers, put_retained_layer);
    });
    bytes
}

fn put_retained_layer(bytes: &mut Vec<u8>, layer: &RetainedLayer) {
    put_u64(bytes, layer.index);
    put_vec(bytes, &layer.regions, |bytes, region| {
        put_u64(bytes, region.id);
        put_vec(bytes, &region.surfaces, |bytes, surface| {
            bytes.push(surface.kind);
            put_expolygon(bytes, &surface.expolygon);
        });
    });
}

pub(in crate::project_slice::tests) fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(in crate::project_slice::tests) fn semantic_identity(bytes: &[u8]) -> (usize, String) {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut entries = std::collections::BTreeMap::new();
    for index in 0..zip.len() {
        let mut file = zip.by_index(index).unwrap();
        if file.is_dir() {
            continue;
        }
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut body).unwrap();
        entries.insert(file.name().to_owned(), body);
    }
    let mut framed = Vec::new();
    for (name, body) in entries {
        framed.extend_from_slice(name.as_bytes());
        framed.push(0);
        framed.extend_from_slice(&body);
    }
    (framed.len(), sha256(&framed))
}

pub(super) fn semantic_hash(bytes: &[u8]) -> String {
    semantic_identity(bytes).1
}

pub(super) fn render_j(stream: &JStream, metadata: &[(&str, [usize; 6])]) -> String {
    let mut text = format!("magic=ARES22J\\0 objects={}\n", stream.objects.len());
    for (object, &(label, [fast, complex, diff, intersection, forward, closing])) in
        stream.objects.iter().zip(metadata)
    {
        text += &format!(
            "object={} label={label} transform={} planned={} sidecar_volumes={} retained_layers={}\n",
            object.source_object_index,
            object.transform_index,
            object.planned_layer_count,
            object.sidecars.len(),
            object.retained_layers.len()
        );
        text += &format!(
            " trace fast={fast} complex={complex} diff={diff} intersection={intersection} forward={forward} closing={closing}\n"
        );
        for sidecar in &object.sidecars {
            text += &format!(
                " sidecar volume_id={} layers={}\n",
                sidecar.occurrence_id,
                sidecar.layers.len()
            );
            for layer in &sidecar.layers {
                text += &format!(
                    "  layer={} {}\n",
                    layer.index,
                    render_expolygons(&layer.expolygons)
                );
            }
        }
        for layer in &object.retained_layers {
            text += &format!(
                " output layer={} regions={}\n",
                layer.index,
                layer.regions.len()
            );
            text += &layer.regions.iter().map(render_region).collect::<String>();
        }
    }
    text
}

fn render_region(region: &Region) -> String {
    let mut text = format!("  region={} surfaces={}", region.id, region.surfaces.len());
    for (index, surface) in region.surfaces.iter().enumerate() {
        text += &format!(
            " S{index}{{type={} {}}}",
            surface.kind,
            render_expolygons(std::slice::from_ref(&surface.expolygon))
        );
    }
    text + "\n"
}

fn render_expolygons(expolygons: &[ExPolygon]) -> String {
    let mut text = format!("expolygons={}", expolygons.len());
    for (index, expolygon) in expolygons.iter().enumerate() {
        text += &format!(
            " E{index}{{C={} holes={}",
            render_path(&expolygon.contour),
            expolygon.holes.len()
        );
        for (hole_index, hole) in expolygon.holes.iter().enumerate() {
            text += &format!(" H{hole_index}={}", render_path(hole));
        }
        text.push('}');
    }
    text
}

fn render_path(points: &[(i64, i64)]) -> String {
    let points = points
        .iter()
        .map(|(x, y)| format!("({x},{y})"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{points}]")
}

fn put_expolygons(bytes: &mut Vec<u8>, expolygons: &[ExPolygon]) {
    put_vec(bytes, expolygons, put_expolygon);
}

fn put_expolygon(bytes: &mut Vec<u8>, expolygon: &ExPolygon) {
    put_polygon(bytes, &expolygon.contour);
    put_vec(bytes, &expolygon.holes, |bytes, hole| {
        put_polygon(bytes, hole)
    });
}

fn put_polygon(bytes: &mut Vec<u8>, polygon: &[(i64, i64)]) {
    put_u64(bytes, u64::try_from(polygon.len()).unwrap());
    for &(x, y) in polygon {
        bytes.extend_from_slice(&x.to_le_bytes());
        bytes.extend_from_slice(&y.to_le_bytes());
    }
}

fn put_vec<T>(bytes: &mut Vec<u8>, values: &[T], mut put: impl FnMut(&mut Vec<u8>, &T)) {
    put_u64(bytes, u64::try_from(values.len()).unwrap());
    for value in values {
        put(bytes, value);
    }
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Reader<'a>(&'a [u8], usize);

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], magic: &[u8; 8]) -> Self {
        assert_eq!(bytes.get(..8), Some(magic.as_slice()));
        Self(bytes, 8)
    }

    fn count_map<T>(&mut self, mut read: impl FnMut(&mut Self) -> T) -> Vec<T> {
        let count = usize::try_from(self.u64()).unwrap();
        (0..count).map(|_| read(self)).collect()
    }

    fn expolygons(&mut self) -> Vec<ExPolygon> {
        self.count_map(Self::expolygon)
    }

    fn expolygon(&mut self) -> ExPolygon {
        ExPolygon {
            contour: self.polygon(),
            holes: self.count_map(Self::polygon),
        }
    }

    fn polygon(&mut self) -> Vec<(i64, i64)> {
        self.count_map(|reader| (reader.i64(), reader.i64()))
    }

    number!(u64, u64, 8);
    number!(i64, i64, 8);
    number!(u32, u32, 4);

    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }

    fn take<const N: usize>(&mut self) -> [u8; N] {
        let end = self.1.checked_add(N).unwrap();
        let value = self.0[self.1..end].try_into().unwrap();
        self.1 = end;
        value
    }

    fn eof(&self) {
        assert_eq!(self.1, self.0.len(), "checkpoint must end at exact EOF");
    }
}
