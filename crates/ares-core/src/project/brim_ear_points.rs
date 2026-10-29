//! `Metadata/brim_ear_points.txt` import (`bbs_3mf.cpp:217`), the
//! per-object painted brim ear point carrier consumed by
//! `Brim.cpp:354-412 make_brim_ears`.

use std::collections::{BTreeMap, BTreeSet};

use crate::SliceError;
use crate::project::domain::Point3d;

use super::{PackagePath, ProjectArchive, domain::ProjectObject, numeric::atof, numeric::atoi};

const BRIM_EAR_POINTS_PATH: &str = "Metadata/brim_ear_points.txt";

/// `BrimPoint` (`Model.hpp`): model-space position plus the painted ear
/// radius (`head_front_radius`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrimPoint {
    pub position: [f32; 3],
    pub head_front_radius: f32,
}

pub(super) fn load(
    archive: &mut ProjectArchive<'_>,
    paths: &BTreeSet<PackagePath>,
    objects: &mut [ProjectObject],
) -> Result<(), SliceError> {
    let mut matching = paths
        .iter()
        .filter(|path| path.as_str().eq_ignore_ascii_case(BRIM_EAR_POINTS_PATH))
        .collect::<Vec<_>>();
    matching.sort_by_key(|path| archive.entry_index(path));
    let mut points = BTreeMap::new();
    for path in matching {
        let bytes = archive.read(path)?;
        import_text(&String::from_utf8_lossy(&bytes), &mut points);
    }
    // bbs_3mf.cpp:2097-2100: one-based ModelObject ordinal, before plate
    // selection.
    for (index, object) in objects.iter_mut().enumerate() {
        if let Some(value) = points.remove(&((index + 1) as i32)) {
            object.set_brim_points(value);
        }
    }
    Ok(())
}

fn import_text(text: &str, points: &mut BTreeMap<i32, Vec<BrimPoint>>) {
    let text = text.strip_suffix('\n').unwrap_or(text);
    let mut records = text.split('\n');
    // bbs_3mf.cpp:2962-2970: optional version header; only version 0
    // tuples parse, any other version yields no points.
    let mut version = 0;
    if let Some(header) = records.next() {
        if let Some(value) = header.strip_prefix("brim_points_format_version=") {
            version = atoi(value);
        } else {
            import_record(header, version, points);
        }
    }
    for record in records {
        import_record(record, version, points);
    }
}

fn import_record(record: &str, version: i32, points: &mut BTreeMap<i32, Vec<BrimPoint>>) {
    // bbs_3mf.cpp:2979-2995: malformed records are nonfatal.
    let Some((header, values)) = record.split_once('|') else {
        return;
    };
    let Some((_, ordinal)) = header.split_once('=') else {
        return;
    };
    let ordinal = atoi(ordinal);
    if ordinal == 0 || points.contains_key(&ordinal) {
        return;
    }
    if version != 0 {
        points.insert(ordinal, Vec::new());
        return;
    }
    // bbs_3mf.cpp:3003-3007: four-value tuples through atof; short tails
    // read as no-conversion zeros.
    let tokens = values
        .split(' ')
        .filter(|token| !token.is_empty())
        .map(atof)
        .collect::<Vec<_>>();
    let tuples = tokens.chunks(4).map(|chunk| BrimPoint {
        position: [
            chunk.first().copied().unwrap_or(0.0) as f32,
            chunk.get(1).copied().unwrap_or(0.0) as f32,
            chunk.get(2).copied().unwrap_or(0.0) as f32,
        ],
        head_front_radius: chunk.get(3).copied().unwrap_or(0.0) as f32,
    });
    points.insert(ordinal, tuples.collect());
}

impl ProjectObject {
    pub(crate) fn set_brim_points(&mut self, points: Vec<BrimPoint>) {
        self.brim_points = points;
    }

    /// `Brim.cpp:367-381`: painted ear centers mapped into the
    /// mesh-centered slicing space the first-layer `lslices` live in —
    /// model point through the first instance's linear transform; the
    /// emit-stage instance translation already carries the load-time
    /// centering compensation (`mesh_prepare.rs center_shift`), so no
    /// extra `-center_offset` shift applies. The world-space `z>0`
    /// filter evaluates through the full instance transform
    /// (`:372-373`).
    pub(crate) fn brim_ear_centers(&self) -> Vec<([f32; 2], f32)> {
        let instance = &self.instances()[0];
        let mut linear = instance.transform();
        linear.clear_translation_xy();
        self.brim_points
            .iter()
            .filter(|point| {
                let world = instance.transform().transform_point_f32(Point3d::new(
                    f64::from(point.position[0]),
                    f64::from(point.position[1]),
                    f64::from(point.position[2]),
                ));
                world[2] <= 0.0
            })
            .map(|point| {
                let center = linear.transform_point_f32(Point3d::new(
                    f64::from(point.position[0]),
                    f64::from(point.position[1]),
                    0.0,
                ));
                ([center[0], center[1]], point.head_front_radius)
            })
            .collect()
    }
}
