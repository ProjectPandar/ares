use super::super::region_fixture::checkpoint::{
    ExPolygon as WireExPolygon, Region, RetainedLayer, Surface as WireSurface,
};
use super::super::support::{KsrArchive, metadata};
use super::fixture::{MObject, WireReader as Reader, WireResult, parse_m, parser_mutation_fixture};
use crate::{SliceError, slice_project};

use super::super::super::task22n_browser_oracle;

#[test]
fn task22n_parser_accepts_readable_behavioral_fixture() {
    let fixture = parser_mutation_fixture();
    let frame = parse_n(&fixture.bytes).unwrap();
    assert!(frame.predecessor_len > 0);
    assert_eq!(frame.predecessor.len(), frame.objects.len());
    assert_eq!(frame.objects.len(), 1);
    assert_eq!(frame.objects[0].slots.len(), 1);
    let record = frame.objects[0].slots[0].as_ref().unwrap();
    assert_eq!(record.current, [0, 0]);
    assert_eq!(record.compatible, [0]);
    assert_eq!(
        (record.lower, record.upper, record.upper_same),
        (None, None, None)
    );
}

#[test]
fn task22n_parser_rejects_envelope_corruption_and_requires_exact_eof() {
    let fixture = parser_mutation_fixture().bytes;
    for length in [0, 1, 7, 8, 15, fixture.len() - 1] {
        assert!(parse_n(&fixture[..length]).is_err(), "length {length}");
    }

    let mut wrong_magic = fixture.clone();
    wrong_magic[0] ^= 1;
    assert!(parse_n(&wrong_magic).is_err());

    let mut oversized_predecessor = fixture.clone();
    oversized_predecessor[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(parse_n(&oversized_predecessor).is_err());

    let mut trailing = fixture;
    trailing.push(0);
    assert!(parse_n(&trailing).is_err());
}

#[test]
fn task22n_parser_rejects_noncanonical_fields_and_impossible_counts() {
    let fixture = parser_mutation_fixture();
    assert!(parse_n(&fixture.bytes).is_ok());
    for (offset, value) in [
        (fixture.slot_presence, 2),
        (fixture.surface_kind, 3),
        (fixture.dispatch, 2),
    ] {
        let mut corrupted = fixture.bytes.clone();
        corrupted[offset] = value;
        assert!(parse_n(&corrupted).is_err(), "offset {offset}");
    }
    let mut impossible_count = fixture.bytes;
    impossible_count[fixture.object_count..fixture.object_count + 8]
        .copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(parse_n(&impossible_count).is_err());
}

#[tokio::test]
async fn task22n_public_slice_runs_perimeter_preflight_before_incomplete_sink() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\t\"bridge_flow\": \"1\",",
        "\t\"bridge_flow\": \"0\",",
    );
    let project = archive.bytes();
    let expected = SliceError::InvalidInput("invalid Orca option bridge_flow".to_owned());
    assert_eq!(task22n_browser_oracle(&project).unwrap_err(), expected);
    assert_eq!(
        slice_project(project, metadata()).await.unwrap_err(),
        expected
    );
}

#[derive(Debug)]
pub(super) struct NFrame {
    pub(super) predecessor_len: usize,
    pub(super) predecessor: Vec<MObject>,
    pub(super) objects: Vec<NObject>,
}

#[derive(Debug)]
pub(super) struct NObject {
    pub(super) source: u64,
    pub(super) transform: u64,
    pub(super) planned: u64,
    pub(super) slots: Vec<Option<NRecord>>,
}

#[derive(Debug)]
pub(super) struct NRecord {
    pub(super) source: u64,
    pub(super) transform: u64,
    pub(super) planned: u64,
    pub(super) layer: u64,
    pub(super) region: u64,
    pub(super) compatible: Vec<u64>,
    pub(super) current: [usize; 2],
    pub(super) lower: Option<usize>,
    pub(super) upper: Option<usize>,
    pub(super) upper_same: Option<[usize; 2]>,
    pub(super) current_surfaces: Vec<WireSurface>,
    pub(super) lower_slices: Option<Vec<WireExPolygon>>,
    pub(super) upper_slices: Option<Vec<WireExPolygon>>,
    pub(super) upper_same_surfaces: Option<Vec<WireSurface>>,
    pub(super) height: u64,
    pub(super) slice_z: u64,
    pub(super) flows: [WireFlow; 4],
    pub(super) spiral: bool,
    pub(super) rotation: u64,
    pub(super) dispatch: u8,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WireFlow {
    pub(super) fields: [u32; 4],
    pub(super) bridge: bool,
    pub(super) mm3_per_mm: u64,
}

pub(super) fn parse_n(bytes: &[u8]) -> WireResult<NFrame> {
    let mut reader = Reader::new(bytes);
    reader.magic(b"ARES22N\0")?;
    let predecessor_len = reader.usize()?;
    let predecessor = parse_m(reader.bytes(predecessor_len)?)?;
    let objects = reader.list(parse_n_object)?;
    reader.finish()?;
    validate_frame(&predecessor, &objects)?;
    Ok(NFrame {
        predecessor_len,
        predecessor,
        objects,
    })
}

fn parse_n_object(reader: &mut Reader<'_>) -> WireResult<NObject> {
    let source = reader.u64()?;
    let transform = reader.u64()?;
    let planned = reader.u64()?;
    let slots = reader.list(|reader| {
        if reader.boolean()? {
            parse_record(reader).map(Some)
        } else {
            Ok(None)
        }
    })?;
    Ok(NObject {
        source,
        transform,
        planned,
        slots,
    })
}

fn parse_record(reader: &mut Reader<'_>) -> WireResult<NRecord> {
    let source = reader.u64()?;
    let transform = reader.u64()?;
    let planned = reader.u64()?;
    let layer = reader.u64()?;
    let region = reader.u64()?;
    let compatible = reader.list(Reader::u64)?;
    let current = [reader.usize()?, reader.usize()?];
    let lower = reader.optional(Reader::usize)?;
    let upper = reader.optional(Reader::usize)?;
    let upper_same = reader.optional(|reader| Ok([reader.usize()?, reader.usize()?]))?;
    let current_surfaces = reader.surfaces()?;
    let lower_slices = lower.map(|_| reader.expolygons()).transpose()?;
    let upper_slices = upper.map(|_| reader.expolygons()).transpose()?;
    let upper_same_surfaces = upper_same.map(|_| reader.surfaces()).transpose()?;
    let height = reader.u64()?;
    let slice_z = reader.u64()?;
    let flows = (0..4)
        .map(|_| read_flow(reader))
        .collect::<WireResult<Vec<_>>>()?
        .try_into()
        .map_err(|_| ())?;
    let spiral = reader.boolean()?;
    let rotation = reader.u64()?;
    let dispatch = reader.u8()?;
    if dispatch > 1 {
        return Err(());
    }
    Ok(NRecord {
        source,
        transform,
        planned,
        layer,
        region,
        compatible,
        current,
        lower,
        upper,
        upper_same,
        current_surfaces,
        lower_slices,
        upper_slices,
        upper_same_surfaces,
        height,
        slice_z,
        flows,
        spiral,
        rotation,
        dispatch,
    })
}

fn read_flow(reader: &mut Reader<'_>) -> WireResult<WireFlow> {
    Ok(WireFlow {
        fields: [reader.u32()?, reader.u32()?, reader.u32()?, reader.u32()?],
        bridge: reader.boolean()?,
        mm3_per_mm: reader.u64()?,
    })
}

fn validate_frame(predecessor: &[MObject], objects: &[NObject]) -> WireResult<()> {
    if predecessor.len() != objects.len() {
        return Err(());
    }
    for ((before, lslices), object) in predecessor.iter().zip(objects) {
        if (
            before.source_object_index,
            before.transform_index,
            before.planned_layer_count,
        ) != (object.source, object.transform, object.planned)
            || before.retained_layers.len() != object.slots.len()
            || lslices.len() != before.retained_layers.len()
            || usize::try_from(object.planned).map_err(|_| ())? != object.slots.len()
        {
            return Err(());
        }
        for (index, (layer, slot)) in before.retained_layers.iter().zip(&object.slots).enumerate() {
            let region = one_region(layer)?;
            let Some(record) = slot else {
                region.surfaces.is_empty().then_some(()).ok_or(())?;
                continue;
            };
            let lower = index.checked_sub(1);
            let upper = (index + 1 < before.retained_layers.len()).then_some(index + 1);
            let upper_surfaces = upper
                .map(|index| one_region(&before.retained_layers[index]))
                .transpose()?
                .map(|region| &region.surfaces);
            if [record.source, record.transform] != [object.source, object.transform]
                || record.planned != index as u64
                || record.layer != layer.index
                || record.region != region.id
                || record.compatible.as_slice() != [region.id]
                || record.current != [0, index]
                || record.lower != lower
                || record.upper != upper
                || record.upper_same != upper.map(|upper| [0, upper])
                || record.current_surfaces != region.surfaces
                || record.lower_slices.as_ref() != lower.map(|i| &lslices[i])
                || record.upper_slices.as_ref() != upper.map(|i| &lslices[i])
                || record.upper_same_surfaces.as_ref() != upper_surfaces
            {
                return Err(());
            }
            let _ = (
                record.height,
                record.slice_z,
                record.flows,
                record.spiral,
                record.rotation,
                record.dispatch,
            );
        }
    }
    Ok(())
}

fn one_region(layer: &RetainedLayer) -> WireResult<&Region> {
    let [region] = layer.regions.as_slice() else {
        return Err(());
    };
    Ok(region)
}
