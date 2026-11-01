//! `ModelInstance::intersects_bed_exclude_region`'s yes/no path (`Model.cpp:3753-3836` at
//! v2.5.0-beta `345efec64`) for one model part. See `docs/architecture/web-plater-arrange-imex.md`,
//! "A2b2".

use super::BedExcludeRegion;
use crate::{Point3d, Transform3d};

/// Whether the model part with mesh `vertices` / `indices`, placed by `matrix` (`instance matrix *
/// volume matrix`), reaches into `region`'s prism.
pub fn model_part_intersects_region(
    _vertices: &[Point3d],
    _indices: &[[u32; 3]],
    _matrix: Transform3d,
    _region: &BedExcludeRegion,
) -> bool {
    todo!("bed_exclusion::model_part_intersects_region")
}
