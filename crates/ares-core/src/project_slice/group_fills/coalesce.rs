use std::{cmp::Ordering, collections::BTreeSet};

use crate::{ExtrusionRole, ProcessInfillPattern};

use super::{
    GroupedFills, RepresentativeSurface, SurfaceFill, SurfaceFillParams, SurfaceFillPattern,
    params::ProjectedLayer,
};

#[derive(Clone, Copy)]
struct ParamsKey(SurfaceFillParams);

impl PartialEq for ParamsKey {
    fn eq(&self, other: &Self) -> bool {
        compare(&self.0, &other.0) == Ordering::Equal
    }
}

impl Eq for ParamsKey {}

impl PartialOrd for ParamsKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParamsKey {
    fn cmp(&self, other: &Self) -> Ordering {
        compare(&self.0, &other.0)
    }
}

pub(super) fn coalesce(projected: ProjectedLayer<'_>) -> GroupedFills {
    let mut unique = BTreeSet::new();
    for params in projected.params.iter().flatten().copied() {
        unique.insert(ParamsKey(params));
    }
    let mut fills = unique
        .into_iter()
        .enumerate()
        .map(|(idx, key)| PendingFill {
            params: SurfaceFillParams { idx, ..key.0 },
            fill: None,
        })
        .collect::<Vec<_>>();

    for (surface, params) in projected.surfaces.iter().zip(&projected.params) {
        let Some(params) = params else {
            continue;
        };
        let index = fills
            .binary_search_by(|fill| compare(&fill.params, params))
            .expect("projected surface parameter was interned before coalescing");
        let pending = &mut fills[index];
        let (kind, expolygon, thickness, thickness_layers, _, extra_perimeters) =
            surface.as_parts();
        if let Some(fill) = &mut pending.fill {
            fill.expolygons.push(expolygon.clone());
        } else {
            pending.fill = Some(SurfaceFill {
                region_id: projected.region_id,
                representative: RepresentativeSurface::from_parts(
                    kind,
                    thickness,
                    thickness_layers,
                    pending.params.bridge_angle,
                    extra_perimeters,
                ),
                expolygons: vec![expolygon.clone()],
                params: pending.params,
                region_id_group: vec![projected.region_id],
                no_overlap_expolygons: projected.no_overlap_expolygons.to_vec(),
            });
        }
    }

    GroupedFills {
        surface_fills: fills
            .into_iter()
            .map(|fill| {
                fill.fill
                    .expect("every interned parameter retains a projected surface")
            })
            .collect(),
        lock_region_param: projected.lock_region_param,
    }
}

struct PendingFill {
    params: SurfaceFillParams,
    fill: Option<SurfaceFill>,
}

fn compare(left: &SurfaceFillParams, right: &SurfaceFillParams) -> Ordering {
    compare_f32(right.bridge_angle, left.bridge_angle)
        .then_with(|| left.extruder.cmp(&right.extruder))
        .then_with(|| pattern_rank(left.pattern).cmp(&pattern_rank(right.pattern)))
        .then_with(|| compare_f64(left.spacing, right.spacing))
        .then_with(|| compare_f64(left.overlap, right.overlap))
        .then_with(|| compare_f32(left.angle, right.angle))
        .then_with(|| left.fixed_angle.cmp(&right.fixed_angle))
        .then_with(|| compare_f32(left.density, right.density))
        .then_with(|| left.multiline.cmp(&right.multiline))
        .then_with(|| compare_f32(left.anchor_length, right.anchor_length))
        .then_with(|| compare_f32(left.anchor_length_max, right.anchor_length_max))
        .then_with(|| compare_f32(left.flow.width, right.flow.width))
        .then_with(|| compare_f32(left.flow.height, right.flow.height))
        .then_with(|| compare_f32(left.flow.nozzle_diameter, right.flow.nozzle_diameter))
        .then_with(|| left.bridge.cmp(&right.bridge))
        .then_with(|| role_rank(left.extrusion_role).cmp(&role_rank(right.extrusion_role)))
        .then_with(|| compare_f32(left.role_speed, right.role_speed))
        .then_with(|| compare_f32(left.lateral_lattice_angle_1, right.lateral_lattice_angle_1))
        .then_with(|| compare_f32(left.lateral_lattice_angle_2, right.lateral_lattice_angle_2))
        .then_with(|| {
            left.symmetric_infill_y_axis
                .cmp(&right.symmetric_infill_y_axis)
        })
        .then_with(|| compare_f32(left.infill_lock_depth, right.infill_lock_depth))
        .then_with(|| compare_f32(left.skin_infill_depth, right.skin_infill_depth))
        .then_with(|| compare_f32(left.infill_overhang_angle, right.infill_overhang_angle))
        .then_with(|| left.gyroid_optimized.cmp(&right.gyroid_optimized))
}

fn compare_f32(left: f32, right: f32) -> Ordering {
    if left < right {
        Ordering::Less
    } else if left > right {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

fn compare_f64(left: f64, right: f64) -> Ordering {
    if left < right {
        Ordering::Less
    } else if left > right {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

const fn pattern_rank(pattern: SurfaceFillPattern) -> u8 {
    match pattern {
        SurfaceFillPattern::Configured(pattern) => match pattern {
            ProcessInfillPattern::Monotonic => 0,
            ProcessInfillPattern::MonotonicLine => 1,
            ProcessInfillPattern::Rectilinear => 2,
            ProcessInfillPattern::AlignedRectilinear => 3,
            ProcessInfillPattern::ZigZag => 4,
            ProcessInfillPattern::CrossZag => 5,
            ProcessInfillPattern::LockedZag => 6,
            ProcessInfillPattern::Line => 7,
            ProcessInfillPattern::Grid => 8,
            ProcessInfillPattern::Triangles => 9,
            ProcessInfillPattern::TriHexagon => 10,
            ProcessInfillPattern::Cubic => 11,
            ProcessInfillPattern::AdaptiveCubic => 12,
            ProcessInfillPattern::QuarterCubic => 13,
            ProcessInfillPattern::SupportCubic => 14,
            ProcessInfillPattern::Lightning => 15,
            ProcessInfillPattern::Honeycomb => 16,
            ProcessInfillPattern::ThreeDHoneycomb => 17,
            ProcessInfillPattern::LateralHoneycomb => 18,
            ProcessInfillPattern::LateralLattice => 19,
            ProcessInfillPattern::CrossHatch => 20,
            ProcessInfillPattern::TpmsD => 21,
            ProcessInfillPattern::TpmsFk => 22,
            ProcessInfillPattern::Gyroid => 23,
            ProcessInfillPattern::Concentric => 24,
            ProcessInfillPattern::HilbertCurve => 25,
            ProcessInfillPattern::ArchimedeanChords => 26,
            ProcessInfillPattern::OctagramSpiral => 27,
        },
        SurfaceFillPattern::ConcentricInternal => 29,
    }
}

const fn role_rank(role: ExtrusionRole) -> u8 {
    match role {
        ExtrusionRole::None => 0,
        ExtrusionRole::Perimeter => 1,
        ExtrusionRole::ExternalPerimeter => 2,
        ExtrusionRole::OverhangPerimeter => 3,
        ExtrusionRole::InternalInfill => 4,
        ExtrusionRole::SolidInfill => 5,
        ExtrusionRole::TopSolidInfill => 6,
        ExtrusionRole::BottomSurface => 7,
        ExtrusionRole::Ironing => 8,
        ExtrusionRole::BridgeInfill => 9,
        ExtrusionRole::InternalBridgeInfill => 10,
        ExtrusionRole::GapFill => 11,
        ExtrusionRole::Skirt => 12,
        ExtrusionRole::Brim => 13,
        ExtrusionRole::SupportMaterial => 14,
        ExtrusionRole::SupportMaterialInterface => 15,
        ExtrusionRole::SupportTransition => 16,
        ExtrusionRole::WipeTower => 17,
        ExtrusionRole::Custom => 18,
        ExtrusionRole::Mixed => 19,
    }
}
