use crate::project_slice::{
    prepare_infill::surface_type_detection,
    region_slices::{RegionSurface, RegionSurfaceKind},
};

pub(super) fn totals(objects: &[surface_type_detection::PreparedSurfaceTypeObject]) -> [usize; 26] {
    let records = objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .collect::<Vec<_>>();
    let mut output = [0; 26];
    output[0] = objects.len();
    output[1] = objects.iter().map(|object| object.records.len()).sum();
    output[2] = records.len();
    output[3] = records.iter().map(|record| record.perimeters.len()).sum();
    output[4] = records
        .iter()
        .flat_map(|record| &record.perimeters)
        .map(|collection| collection.entities.len())
        .sum();
    output[5] = records.iter().map(|record| record.thin_fills.len()).sum();
    output[6] = records
        .iter()
        .map(|record| record.fill_expolygons.len())
        .sum();
    output[7] = records
        .iter()
        .map(|record| record.fill_no_overlap_expolygons.len())
        .sum();
    let mut slice_solids = 0;
    let mut fill_solids = 0;
    for record in records {
        count_surfaces(&mut output[8..13], &mut slice_solids, &record.slices);
        count_surfaces(&mut output[13..18], &mut fill_solids, &record.fill_surfaces);
        count_geometry(&mut output[18..21], &record.slices);
        count_geometry(&mut output[21..24], &record.fill_surfaces);
    }
    output[24] = slice_solids;
    output[25] = fill_solids;
    output
}

fn count_surfaces(output: &mut [usize], solid: &mut usize, surfaces: &[RegionSurface]) {
    output[0] += surfaces.len();
    for surface in surfaces {
        match surface.as_parts().0 {
            RegionSurfaceKind::Top => output[1] += 1,
            RegionSurfaceKind::Bottom => output[2] += 1,
            RegionSurfaceKind::BottomBridge => output[3] += 1,
            RegionSurfaceKind::Internal => output[4] += 1,
            RegionSurfaceKind::InternalSolid => *solid += 1,
        }
    }
}

fn count_geometry(output: &mut [usize], surfaces: &[RegionSurface]) {
    for surface in surfaces {
        let expolygon = surface.as_parts().1;
        output[0] += 1;
        output[1] += expolygon.holes().len();
        output[2] += std::iter::once(expolygon.contour())
            .chain(expolygon.holes())
            .map(|polygon| polygon.points().len())
            .sum::<usize>();
    }
}
