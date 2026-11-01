//! `GCode.cpp:3077–3113`: flat-print chamber hold uses scaled areas per
//! PrintObject (not per instance), plus its brim and the tower mesh bottom.
use super::{brim::BrimPlan, layers::Schedule};
use crate::{
    SliceError,
    geometry::{CoordinateScale, Polygon},
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        support_material::InstalledSupportLayer,
    },
};

pub(super) fn hold_temperature(
    traversal: &PreparedPostClassicTraversal,
    support: &[Vec<InstalledSupportLayer>],
    schedule: &Schedule,
    brim: &[Option<BrimPlan>],
) -> Result<bool, SliceError> {
    let settings = &traversal.resolved.views.full;
    let first_z = settings.process.print.initial_layer_print_height.0;
    let mut height = -1.0_f64;
    let mut area = 0.0;
    for (index, object) in traversal.objects.iter().enumerate() {
        let layers = object.physical_layers();
        if let Some(last) = layers.last() {
            height = height.max(last.print_z);
        }
        if layers
            .first()
            .is_some_and(|layer| layer.print_z < first_z + 1e-4)
        {
            for polygon in object.slices(0).into_iter().flatten() {
                area += polygon.area();
            }
        }
        let installed = support[index].first();
        let raft = schedule
            .raft
            .as_ref()
            .and_then(|raft| raft.plans[index].first());
        let first_raft =
            raft.filter(|raft| installed.is_none_or(|layer| raft.z.print_z < layer.print_z));
        if let Some(raft) = first_raft.filter(|raft| raft.z.print_z < first_z + 1e-4) {
            for polygon in &raft.polygons {
                area += polygon.area();
            }
        } else if first_raft.is_none()
            && let Some(layer) = installed.filter(|layer| layer.print_z < first_z + 1e-4)
        {
            for polygon in &layer.support_islands {
                area += polygon.area();
            }
        }
        if let Some(brim) = &brim[index] {
            add_brim_area(&mut area, brim, &schedule.instances[index], traversal.scale)?;
        }
    }
    if let Some(tower) = &schedule.tower {
        area += tower
            .bottom_polygon(settings, traversal.scale)
            .map_err(|error| {
                SliceError::InvalidInput(format!("wipe tower mesh bottom: {error:?}"))
            })?
            .area();
    }
    Ok(height > 0.0 && height < 0.3 && area > 40000.0 * 1.0e10)
}

fn add_brim_area(
    area: &mut f64,
    brim: &BrimPlan,
    copies: &[super::layers::Instance],
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    // Brim.cpp:1004-1010: uncombined per-instance paths already carry their
    // instance shifts (plate coordinates); combined/global plans keep the
    // copy translation below.
    if !brim.instances.is_empty() {
        for instance in &brim.instances {
            for path in &instance.paths {
                *area += spacing_coverage(path, brim.spacing, scale)?
                    .iter()
                    .map(|polygon| polygon.area())
                    .sum::<f64>();
            }
        }
        return Ok(());
    }
    // The temporary single-object brim shell emits each copy separately.
    for copy in copies {
        for path in &brim.paths {
            let translated = path
                .iter()
                .map(|point| {
                    crate::geometry::Point::new(
                        point.x() + copy.shift.x(),
                        point.y() + copy.shift.y(),
                    )
                })
                .collect::<Vec<_>>();
            for polygon in spacing_coverage(&translated, brim.spacing, scale)? {
                *area += polygon.area();
            }
        }
    }
    Ok(())
}

fn spacing_coverage(
    points: &[crate::geometry::Point],
    spacing: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, SliceError> {
    super::footprint::covered_by_spacing(points.iter().copied(), spacing, 0.0, scale)
        .map_err(|error| SliceError::InvalidInput(format!("brim spacing coverage: {error:?}")))
}
