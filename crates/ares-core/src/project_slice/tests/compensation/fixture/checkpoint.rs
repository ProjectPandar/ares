use super::super::super::region_fixture::checkpoint::{
    ExPolygon as WireExPolygon, GeometryLayer, JObject, Region, RetainedLayer, Sidecar, Surface,
};

pub(super) type MObject = (JObject, Vec<Vec<WireExPolygon>>);

pub(super) fn parse_m(bytes: &[u8]) -> Vec<MObject> {
    let mut reader = MReader(bytes, 0);
    assert_eq!(reader.take::<8>(), *b"ARES22M\0");
    let objects = reader.list(|reader| {
        let source_object_index = reader.u64();
        let transform_index = reader.u64();
        let planned_layer_count = reader.u64();
        let sidecars = reader.list(|reader| Sidecar {
            occurrence_id: reader.u64(),
            layers: reader.list(|reader| GeometryLayer {
                index: reader.u64(),
                expolygons: reader.expolygons(),
            }),
        });
        let mut lslices = Vec::new();
        let retained_layers = reader.list(|reader| {
            let layer = RetainedLayer {
                index: reader.u64(),
                regions: reader.list(|reader| Region {
                    id: reader.u64(),
                    surfaces: reader.list(|reader| {
                        let kind = reader.u8();
                        assert_eq!(kind, 4);
                        Surface {
                            kind,
                            expolygon: reader.expolygon(),
                        }
                    }),
                }),
            };
            lslices.push(reader.expolygons());
            layer
        });
        (
            JObject {
                source_object_index,
                transform_index,
                planned_layer_count,
                sidecars,
                retained_layers,
            },
            lslices,
        )
    });
    assert_eq!(reader.1, bytes.len(), "checkpoint must end at exact EOF");
    objects
}

struct MReader<'a>(&'a [u8], usize);

impl MReader<'_> {
    fn list<T>(&mut self, mut read: impl FnMut(&mut Self) -> T) -> Vec<T> {
        (0..self.u64()).map(|_| read(self)).collect()
    }
    fn expolygons(&mut self) -> Vec<WireExPolygon> {
        self.list(Self::expolygon)
    }
    fn expolygon(&mut self) -> WireExPolygon {
        WireExPolygon {
            contour: self.list(|reader| (reader.i64(), reader.i64())),
            holes: self.list(|reader| reader.list(|reader| (reader.i64(), reader.i64()))),
        }
    }
    fn u64(&mut self) -> u64 {
        u64::from_le_bytes(self.take())
    }
    fn i64(&mut self) -> i64 {
        i64::from_le_bytes(self.take())
    }
    fn u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let end = self.1.checked_add(N).unwrap();
        let value = self.0.get(self.1..end).unwrap().try_into().unwrap();
        self.1 = end;
        value
    }
}

pub(super) fn surface_geometry(regions: &[Region]) -> Vec<WireExPolygon> {
    regions[0]
        .surfaces
        .iter()
        .map(|surface| surface.expolygon.clone())
        .collect()
}

pub(super) fn assert_same_geometry(actual: &[WireExPolygon], expected: &[WireExPolygon]) {
    assert_eq!(actual.len(), expected.len());
    assert!(actual.iter().all(|expolygon| expected.contains(expolygon)));
}

pub(super) fn expolygon(contour: &[(i64, i64)]) -> WireExPolygon {
    WireExPolygon {
        contour: contour.to_vec(),
        holes: Vec::new(),
    }
}
