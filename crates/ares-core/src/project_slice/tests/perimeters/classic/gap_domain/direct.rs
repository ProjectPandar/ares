use crate::{
    geometry::{JoinType, difference_ex, offset2_ex, opening_ex},
    project_slice::perimeters::prepare_post_classic_gap_domain,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o11_direct_empty_gate_bounds_casts_and_geometry_match_source() {
    let prepared = prepare_post_classic_gap_domain(ksr_project()).unwrap();
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    let mut empty = 0;
    let mut nonempty = 0;
    for (object, traversal) in prepared.objects.iter().zip(&prepared.predecessor.objects) {
        let onion = &traversal.predecessor.predecessor;
        let prelude = &onion.predecessor.predecessor;
        assert_eq!(object.records.len(), onion.records.len());
        assert_eq!(object.records.len(), prelude.records.len());
        for ((output, onion), prelude) in object
            .records
            .iter()
            .zip(&onion.records)
            .zip(&prelude.records)
        {
            let (Some(output), Some(onion), Some(prelude)) = (output, onion, prelude) else {
                assert!(output.is_none() && onion.is_none() && prelude.is_none());
                continue;
            };
            assert_eq!(output.surfaces.len(), onion.surfaces.len());
            assert_eq!(output.surfaces.len(), prelude.surfaces.len());
            for ((output, onion), source) in output
                .surfaces
                .iter()
                .zip(&onion.surfaces)
                .zip(&prelude.surfaces)
            {
                let has_gap = verify_surface(
                    output,
                    &onion.gaps,
                    [onion.source_index, source.source_index],
                    (
                        prelude.perimeter_width,
                        prelude.external_width,
                        prelude.perimeter_spacing,
                    ),
                    prelude.surface_simplify_resolution,
                );
                nonempty += usize::from(has_gap);
                empty += usize::from(!has_gap);
            }
        }
    }
    assert!(empty > 0);
    assert!(nonempty > 0);
}

fn verify_surface(
    output: &crate::project_slice::perimeters::classic::gap_domain::PreparedGapDomainSurface,
    gaps: &[crate::geometry::ExPolygon],
    source_indices: [usize; 2],
    widths: (i64, i64, i64),
    resolution: f64,
) -> bool {
    assert_eq!(output.source_index, source_indices[0]);
    assert_eq!(output.source_index, source_indices[1]);
    if gaps.is_empty() {
        assert!(output.pre_medial.is_none());
        return false;
    }

    let output = output.pre_medial.as_ref().unwrap();
    let (perimeter_width, external_width, perimeter_spacing) = widths;
    let min = 0.2_f64 * perimeter_width.min(external_width) as f64 * (1.0_f64 - 0.4_f64);
    let max = 2.0_f64 * perimeter_spacing as f64;
    assert_eq!((output.min, output.max), (min, max));
    let opened = opening_ex(gaps, (min / 2.0) as f32, JoinType::Miter, 3.0).unwrap();
    let offset = offset2_ex(
        gaps,
        -((max / 2.0) as f32),
        (max / 2.0 + 10.0_f64) as f32,
        JoinType::Miter,
        3.0,
    )
    .unwrap();
    let mut expected = difference_ex(&opened, &offset).unwrap();
    for expolygon in &mut expected {
        expolygon.douglas_peucker(resolution);
    }
    assert_eq!(output.expolygons, expected);
    true
}
