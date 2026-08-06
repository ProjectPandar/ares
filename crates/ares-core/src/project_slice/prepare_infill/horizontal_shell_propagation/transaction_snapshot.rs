use crate::{
    geometry::{ExPolygon, Polygon},
    project_slice::prepare_infill::horizontal_shell_promotion::PreparedPostHorizontalShellPromotion,
};

pub(super) fn fingerprint(prepared: &PreparedPostHorizontalShellPromotion) -> i128 {
    let mut digest = 0x4f26_524f_4c4c_4241_434b_5f47_5241_5048_i128;
    mix(
        &mut digest,
        std::ptr::from_ref(prepared.predecessor.as_ref()) as usize,
    );
    for pointer in [
        prepared.objects.as_ptr() as usize,
        prepared.caches.as_ptr() as usize,
        prepared.projections.as_ptr() as usize,
        prepared.trims.as_ptr() as usize,
        prepared.regularizations.as_ptr() as usize,
        prepared.filters.as_ptr() as usize,
    ] {
        mix(&mut digest, pointer);
    }
    for object in &prepared.objects {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in &object.records {
            mix(&mut digest, usize::from(record.is_some()));
            let Some(record) = record else { continue };
            for values in [
                (record.perimeters.as_ptr() as usize, record.perimeters.len()),
                (record.thin_fills.as_ptr() as usize, record.thin_fills.len()),
                (record.slices.as_ptr() as usize, record.slices.len()),
                (
                    record.fill_surfaces.as_ptr() as usize,
                    record.fill_surfaces.len(),
                ),
                (
                    record.fill_expolygons.as_ptr() as usize,
                    record.fill_expolygons.len(),
                ),
                (
                    record.fill_no_overlap_expolygons.as_ptr() as usize,
                    record.fill_no_overlap_expolygons.len(),
                ),
            ] {
                mix(&mut digest, values.0);
                mix(&mut digest, values.1);
            }
            for surface in record.slices.iter().chain(&record.fill_surfaces) {
                let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
                mix(&mut digest, kind as usize);
                mix(&mut digest, thickness.to_bits() as usize);
                mix(&mut digest, usize::from(layers));
                mix(&mut digest, angle.to_bits() as usize);
                mix(&mut digest, usize::from(extra));
                expolygon_fingerprint(&mut digest, expolygon);
            }
            for expolygon in record
                .fill_expolygons
                .iter()
                .chain(&record.fill_no_overlap_expolygons)
            {
                expolygon_fingerprint(&mut digest, expolygon);
            }
        }
    }
    for object in &prepared.caches {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in object.records.iter().flatten() {
            for paths in [&record.top_surfaces, &record.bottom_surfaces, &record.holes] {
                paths_fingerprint(&mut digest, paths);
            }
        }
    }
    for object in &prepared.projections {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in object.records.iter().flatten() {
            paths_fingerprint(&mut digest, &record.shell);
            paths_fingerprint(&mut digest, &record.holes);
        }
    }
    for object in &prepared.trims {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in object.records.iter().flatten() {
            paths_fingerprint(&mut digest, &record.shell);
        }
    }
    for object in &prepared.regularizations {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in object.records.iter().flatten() {
            expolygons_fingerprint(&mut digest, &record.regularized_shell);
        }
    }
    for object in &prepared.filters {
        mix(&mut digest, object.records.as_ptr() as usize);
        for record in object.records.iter().flatten() {
            expolygons_fingerprint(&mut digest, &record.filtered_shell);
        }
    }
    digest
}

fn expolygons_fingerprint(digest: &mut i128, values: &[ExPolygon]) {
    mix(digest, values.as_ptr() as usize);
    mix(digest, values.len());
    for value in values {
        expolygon_fingerprint(digest, value);
    }
}

fn expolygon_fingerprint(digest: &mut i128, value: &ExPolygon) {
    path_fingerprint(digest, value.contour());
    paths_fingerprint(digest, value.holes());
}

fn paths_fingerprint(digest: &mut i128, values: &[Polygon]) {
    mix(digest, values.as_ptr() as usize);
    mix(digest, values.len());
    for value in values {
        path_fingerprint(digest, value);
    }
}

fn path_fingerprint(digest: &mut i128, value: &Polygon) {
    mix(digest, value.points().as_ptr() as usize);
    mix(digest, value.points().len());
    for point in value.points() {
        *digest = digest
            .wrapping_mul(1099511628211)
            .wrapping_add(i128::from(point.x()));
        *digest = digest
            .wrapping_mul(1099511628211)
            .wrapping_add(i128::from(point.y()));
    }
}

fn mix(digest: &mut i128, value: usize) {
    *digest = digest
        .wrapping_mul(1099511628211)
        .wrapping_add(value as i128);
}
