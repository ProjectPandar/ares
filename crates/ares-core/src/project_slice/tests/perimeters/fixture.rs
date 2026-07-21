use crate::{
    FloatOrPercent, ObjectOptions, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, Percent,
    RegionOptions,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        compensation::{PostCompensationPrintObject, apply_project_compensation},
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
        task22m_oracle,
    },
};

use super::super::region_fixture::checkpoint::{
    ExPolygon as WireExPolygon, GeometryLayer, JObject, Region, RetainedLayer, Sidecar,
    Surface as WireSurface,
};
use super::super::support::{identity_resolved, object_options, region};

mod archive;
mod context_pairs;
mod flow_pairs;

pub(super) struct Case {
    pub(super) object: PostCompensationPrintObject,
    pub(super) resolved: ResolvedProjectObject,
}

#[derive(Debug, PartialEq)]
pub(super) struct Snapshot {
    checkpoint: Vec<u8>,
    layers: Vec<Vec<[u64; 4]>>,
    objects: Vec<ObjectOptions>,
    regions: Vec<Vec<RegionOptions>>,
}

pub(super) fn flow_options() -> (RegionOptions, ObjectOptions) {
    let mut region = region();
    region.outer_wall_line_width = FloatOrPercent::Float(0.42);
    region.inner_wall_line_width = FloatOrPercent::Float(0.45);
    region.internal_solid_infill_line_width = FloatOrPercent::Float(0.42);
    region.bridge_line_width = FloatOrPercent::Percent(Percent(100.0));
    region.bridge_flow = OrcaFloat(1.0);
    region.outer_wall_filament_id = OrcaInt(1);
    region.inner_wall_filament_id = OrcaInt(1);
    region.internal_solid_filament_id = OrcaInt(1);
    let mut object = object_options();
    object.line_width = FloatOrPercent::Float(0.42);
    object.thick_bridges = OrcaBool(false);
    (region, object)
}

pub(super) fn case(
    source_object_index: usize,
    region: RegionOptions,
    object: ObjectOptions,
    layers: &[(f64, usize)],
    scale: CoordinateScale,
) -> Case {
    let mut resolved = identity_resolved(source_object_index);
    resolved.object = object;
    let mut print_z = 0.0;
    let mut planned = Vec::with_capacity(layers.len());
    let mut region_layers = Vec::with_capacity(layers.len());
    for (id, &(height, surface_count)) in layers.iter().enumerate() {
        let slice_z = print_z + 0.5 * height;
        print_z += height;
        planned.push(PlannedLayer {
            id,
            height,
            print_z,
            slice_z,
        });
        region_layers.push(RegionLayer {
            surfaces: (0..surface_count)
                .map(|surface| RegionSurface::internal(rectangle(source_object_index, id, surface)))
                .collect(),
        });
    }
    let post_region = PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index: 0,
            layers: planned,
        },
        volume_slices: Vec::new(),
        regions: vec![PostRegion {
            id: 0,
            options: region,
            layers: region_layers,
        }],
    };
    let mut objects = apply_project_compensation(
        vec![post_region],
        std::slice::from_ref(&resolved),
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
        scale,
    )
    .unwrap();
    Case {
        object: objects.pop().unwrap(),
        resolved,
    }
}

pub(super) fn split(
    cases: Vec<Case>,
) -> (Vec<PostCompensationPrintObject>, Vec<ResolvedProjectObject>) {
    cases
        .into_iter()
        .map(|case| (case.object, case.resolved))
        .unzip()
}

pub(super) fn snapshot(
    objects: &[PostCompensationPrintObject],
    resolved: &[ResolvedProjectObject],
) -> Snapshot {
    let mut layers = Vec::with_capacity(objects.len());
    let mut regions = Vec::with_capacity(objects.len());
    for object in objects {
        let (post_region, _) = object.as_parts();
        let (plan, _, object_regions) = post_region.as_parts();
        layers.push(
            plan.layers
                .iter()
                .map(|layer| {
                    [
                        layer.id as u64,
                        layer.height.to_bits(),
                        layer.print_z.to_bits(),
                        layer.slice_z.to_bits(),
                    ]
                })
                .collect(),
        );
        regions.push(
            object_regions
                .iter()
                .map(|region| region.as_parts().1.clone())
                .collect(),
        );
    }
    Snapshot {
        checkpoint: task22m_oracle::encode(objects),
        layers,
        objects: resolved.iter().map(|item| item.object.clone()).collect(),
        regions,
    }
}

fn rectangle(source: usize, layer: usize, surface: usize) -> ExPolygon {
    let base = (source * 1_000 + layer * 100 + surface * 10) as i64;
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(base, base),
            Point::new(base + 8, base),
            Point::new(base + 8, base + 6),
            Point::new(base, base + 6),
        ]),
        Vec::new(),
    )
}

pub(super) type WireResult<T> = Result<T, ()>;
pub(super) type MObject = (JObject, Vec<Vec<WireExPolygon>>);

pub(super) fn parse_m(bytes: &[u8]) -> WireResult<Vec<MObject>> {
    let mut reader = WireReader::new(bytes);
    reader.magic(b"ARES22M\0")?;
    let objects = reader.list(|reader| {
        let source_object_index = reader.u64()?;
        let transform_index = reader.u64()?;
        let planned_layer_count = reader.u64()?;
        let sidecars = reader.list(|reader| {
            Ok(Sidecar {
                occurrence_id: reader.u64()?,
                layers: reader.list(|reader| {
                    Ok(GeometryLayer {
                        index: reader.u64()?,
                        expolygons: reader.expolygons()?,
                    })
                })?,
            })
        })?;
        let mut lslices = Vec::new();
        let retained_layers = reader.list(|reader| {
            let index = reader.u64()?;
            let regions = reader.list(|reader| {
                Ok(Region {
                    id: reader.u64()?,
                    surfaces: reader.surfaces()?,
                })
            })?;
            lslices.push(reader.expolygons()?);
            Ok(RetainedLayer { index, regions })
        })?;
        Ok((
            JObject {
                source_object_index,
                transform_index,
                planned_layer_count,
                sidecars,
                retained_layers,
            },
            lslices,
        ))
    })?;
    reader.finish()?;
    Ok(objects)
}

pub(super) struct ParserMutationFixture {
    pub(super) bytes: Vec<u8>,
    pub(super) object_count: usize,
    pub(super) slot_presence: usize,
    pub(super) surface_kind: usize,
    pub(super) dispatch: usize,
}

pub(super) fn parser_mutation_fixture() -> ParserMutationFixture {
    let mut predecessor = b"ARES22M\0".to_vec();
    for value in [1, 0, 0, 1, 0, 1, 0, 1, 0, 1] {
        put_u64(&mut predecessor, value);
    }
    predecessor.push(4);
    for value in [0; 3] {
        put_u64(&mut predecessor, value);
    }

    let mut bytes = b"ARES22N\0".to_vec();
    put_u64(&mut bytes, predecessor.len() as u64);
    bytes.extend_from_slice(&predecessor);
    let object_count = bytes.len();
    put_u64(&mut bytes, 1);
    for value in [0, 0, 1, 1] {
        put_u64(&mut bytes, value);
    }
    let slot_presence = bytes.len();
    bytes.push(1);
    for value in [0, 0, 0, 0, 0, 1, 0, 0, 0] {
        put_u64(&mut bytes, value);
    }
    bytes.extend_from_slice(&[0, 0, 0]);
    put_u64(&mut bytes, 1);
    let surface_kind = bytes.len();
    bytes.push(4);
    put_u64(&mut bytes, 0);
    put_u64(&mut bytes, 0);
    put_u64(&mut bytes, 0);
    put_u64(&mut bytes, 0);
    for _ in 0..4 {
        for _ in 0..4 {
            put_u32(&mut bytes, 0);
        }
        bytes.push(0);
        put_u64(&mut bytes, 0);
    }
    bytes.push(0);
    put_u64(&mut bytes, 0);
    let dispatch = bytes.len();
    bytes.push(0);
    ParserMutationFixture {
        bytes,
        object_count,
        slot_presence,
        surface_kind,
        dispatch,
    }
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) struct WireReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> WireReader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(super) fn finish(self) -> WireResult<()> {
        (self.offset == self.bytes.len()).then_some(()).ok_or(())
    }

    pub(super) fn magic(&mut self, magic: &[u8; 8]) -> WireResult<()> {
        (self.bytes(8)? == magic).then_some(()).ok_or(())
    }

    pub(super) fn list<T>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> WireResult<T>,
    ) -> WireResult<Vec<T>> {
        let count = self.usize()?;
        if count > self.bytes.len().saturating_sub(self.offset) {
            return Err(());
        }
        (0..count).map(|_| read(self)).collect()
    }

    pub(super) fn optional<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> WireResult<T>,
    ) -> WireResult<Option<T>> {
        self.boolean()?.then(|| read(self)).transpose()
    }

    pub(super) fn surfaces(&mut self) -> WireResult<Vec<WireSurface>> {
        self.list(|reader| {
            let kind = reader.u8()?;
            if kind != 4 {
                return Err(());
            }
            Ok(WireSurface {
                kind,
                expolygon: reader.expolygon()?,
            })
        })
    }

    pub(super) fn expolygons(&mut self) -> WireResult<Vec<WireExPolygon>> {
        self.list(Self::expolygon)
    }

    fn expolygon(&mut self) -> WireResult<WireExPolygon> {
        Ok(WireExPolygon {
            contour: self.list(|reader| Ok((reader.i64()?, reader.i64()?)))?,
            holes: self.list(|reader| reader.list(|reader| Ok((reader.i64()?, reader.i64()?))))?,
        })
    }

    pub(super) fn boolean(&mut self) -> WireResult<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(()),
        }
    }

    pub(super) fn usize(&mut self) -> WireResult<usize> {
        usize::try_from(self.u64()?).map_err(|_| ())
    }

    pub(super) fn u64(&mut self) -> WireResult<u64> {
        Ok(u64::from_le_bytes(self.take()?))
    }

    fn i64(&mut self) -> WireResult<i64> {
        Ok(i64::from_le_bytes(self.take()?))
    }

    pub(super) fn u32(&mut self) -> WireResult<u32> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    pub(super) fn u8(&mut self) -> WireResult<u8> {
        Ok(self.bytes(1)?[0])
    }

    fn take<const N: usize>(&mut self) -> WireResult<[u8; N]> {
        self.bytes(N)?.try_into().map_err(|_| ())
    }

    pub(super) fn bytes(&mut self, len: usize) -> WireResult<&'a [u8]> {
        let end = self.offset.checked_add(len).ok_or(())?;
        let value = self.bytes.get(self.offset..end).ok_or(())?;
        self.offset = end;
        Ok(value)
    }
}
