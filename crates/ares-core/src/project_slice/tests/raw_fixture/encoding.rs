use crate::{
    ProjectVolumeType,
    mesh_slicer::{EndpointReference, FacetEdgeType, IntersectionLine},
};

const PREFIX: &[u8; 26] = b"ares-task22b-raw-state-v1\0";

pub(super) const VERTEX: u8 = 0;
pub(super) const EDGE: u8 = 1;
pub(super) const GENERAL: u8 = 0;
pub(super) const TOP: u8 = 1;

pub(super) struct ObjectView<'a> {
    pub(super) source_object_index: usize,
    pub(super) transform_index: usize,
    pub(super) volumes: Vec<VolumeView<'a>>,
}

pub(super) struct VolumeView<'a> {
    pub(super) ordinal: u32,
    pub(super) volume_type: ProjectVolumeType,
    pub(super) layers: &'a [Vec<IntersectionLine>],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct LineRecord {
    a_x: i64,
    a_y: i64,
    a_kind: u8,
    a_id: u32,
    b_x: i64,
    b_y: i64,
    b_kind: u8,
    b_id: u32,
    edge_type: u8,
}

impl LineRecord {
    pub(super) const fn new(a: (i64, i64, u8, u32), b: (i64, i64, u8, u32), edge_type: u8) -> Self {
        Self {
            a_x: a.0,
            a_y: a.1,
            a_kind: a.2,
            a_id: a.3,
            b_x: b.0,
            b_y: b.1,
            b_kind: b.2,
            b_id: b.3,
            edge_type,
        }
    }

    fn write(self, output: &mut Vec<u8>) {
        output.extend_from_slice(&self.a_x.to_be_bytes());
        output.extend_from_slice(&self.a_y.to_be_bytes());
        output.push(self.a_kind);
        output.extend_from_slice(&self.a_id.to_be_bytes());
        output.extend_from_slice(&self.b_x.to_be_bytes());
        output.extend_from_slice(&self.b_y.to_be_bytes());
        output.push(self.b_kind);
        output.extend_from_slice(&self.b_id.to_be_bytes());
        output.push(self.edge_type);
    }
}

pub(super) fn line_record(line: IntersectionLine) -> LineRecord {
    let a = line.a();
    let b = line.b();
    let a_point = a.point();
    let b_point = b.point();
    let (a_kind, a_id) = reference_tag(a.reference());
    let (b_kind, b_id) = reference_tag(b.reference());
    LineRecord::new(
        (a_point.x(), a_point.y(), a_kind, a_id),
        (b_point.x(), b_point.y(), b_kind, b_id),
        edge_type_tag(line.edge_type()),
    )
}

pub(super) fn sorted_records(lines: &[IntersectionLine]) -> Vec<LineRecord> {
    let mut records = lines.iter().copied().map(line_record).collect::<Vec<_>>();
    records.sort_unstable();
    records
}

pub(super) fn encode(objects: &[ObjectView<'_>], semantic_order: bool) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(PREFIX);
    write_u32(&mut output, objects.len());
    for object in objects {
        output.extend_from_slice(
            &u64::try_from(object.source_object_index)
                .unwrap()
                .to_be_bytes(),
        );
        output.extend_from_slice(&u64::try_from(object.transform_index).unwrap().to_be_bytes());
        write_u32(&mut output, object.volumes.len());
        for volume in &object.volumes {
            encode_volume(&mut output, volume, semantic_order);
        }
    }
    output
}

fn encode_volume(output: &mut Vec<u8>, volume: &VolumeView<'_>, semantic_order: bool) {
    output.extend_from_slice(&volume.ordinal.to_be_bytes());
    output.push(volume_type_tag(volume.volume_type));
    write_u32(output, volume.layers.len());
    for (layer_index, lines) in volume.layers.iter().enumerate() {
        encode_layer(output, layer_index, lines, semantic_order);
    }
}

fn encode_layer(
    output: &mut Vec<u8>,
    layer_index: usize,
    lines: &[IntersectionLine],
    semantic_order: bool,
) {
    write_u32(output, layer_index);
    write_u32(output, lines.len());
    let mut records = lines.iter().copied().map(line_record).collect::<Vec<_>>();
    if semantic_order {
        records.sort_unstable();
    }
    for record in records {
        record.write(output);
    }
}

fn write_u32(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
}

fn reference_tag(reference: EndpointReference) -> (u8, u32) {
    match reference {
        EndpointReference::Vertex(id) => (VERTEX, id),
        EndpointReference::Edge(id) => (EDGE, id),
    }
}

fn edge_type_tag(edge_type: FacetEdgeType) -> u8 {
    match edge_type {
        FacetEdgeType::General => GENERAL,
        FacetEdgeType::Top => TOP,
    }
}

fn volume_type_tag(volume_type: ProjectVolumeType) -> u8 {
    match volume_type {
        ProjectVolumeType::ModelPart => 0,
        ProjectVolumeType::NegativeVolume => 1,
        ProjectVolumeType::ParameterModifier => 2,
        ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker => {
            unreachable!("excluded support volume reached raw-state encoding")
        }
    }
}
