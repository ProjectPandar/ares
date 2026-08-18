mod render;
#[cfg(test)]
mod tests;
mod types;

use render::{encode_layer_geometry, encode_layer_metadata, encode_layer_table};
pub(super) use types::{
    EncodedOracle, OracleFlow, OracleGroup, OracleLayer, OracleLockCounts, OracleParams,
    OracleRepresentative, OracleStage, OracleTotals,
};

pub(super) fn encode(layers: &[OracleLayer<'_>]) -> EncodedOracle {
    let mut ordered_layers = layers.iter().collect::<Vec<_>>();
    ordered_layers.sort_unstable_by_key(|layer| layer.layer_id);

    let mut metadata = String::new();
    let mut geometry_records = Vec::new();
    let mut layer_table =
        String::from("layer\tgroups\texpolygons\tholes\tpoints\tkinds\tpatterns\troles\n");
    for layer in ordered_layers {
        encode_layer_metadata(&mut metadata, layer);
        encode_layer_geometry(&mut geometry_records, layer);
        encode_layer_table(&mut layer_table, layer);
    }

    // ASCII-only records make Rust byte ordering identical to `LC_ALL=C sort`.
    geometry_records.sort_unstable();
    let mut canonical_geometry = Vec::new();
    for record in geometry_records {
        canonical_geometry.extend_from_slice(record.as_bytes());
        canonical_geometry.push(b'\n');
    }

    EncodedOracle {
        metadata: metadata.into_bytes(),
        canonical_geometry,
        layer_table: layer_table.into_bytes(),
    }
}

pub(super) fn totals(layers: &[OracleLayer<'_>]) -> OracleTotals {
    let mut totals = OracleTotals {
        layers: layers.len(),
        groups: 0,
        fill_expolygons: 0,
        fill_holes: 0,
        fill_paths: 0,
        fill_points: 0,
        no_overlap_expolygons: 0,
        nonempty_layers: 0,
        empty_layers: 0,
    };
    for layer in layers {
        totals.groups += layer.groups.len();
        if layer.groups.is_empty() {
            totals.empty_layers += 1;
        } else {
            totals.nonempty_layers += 1;
        }
        for group in &layer.groups {
            totals.fill_expolygons += group.fills.len();
            totals.no_overlap_expolygons += group.no_overlap.len();
            for expolygon in group.fills {
                totals.fill_holes += expolygon.holes().len();
                totals.fill_paths += 1 + expolygon.holes().len();
                totals.fill_points += expolygon.contour().points().len()
                    + expolygon
                        .holes()
                        .iter()
                        .map(|hole| hole.points().len())
                        .sum::<usize>();
            }
        }
    }
    totals
}

pub(super) const fn configured_pattern_rank(pattern: crate::ProcessInfillPattern) -> u8 {
    use crate::ProcessInfillPattern as Pattern;

    match pattern {
        Pattern::Monotonic => 0,
        Pattern::MonotonicLine => 1,
        Pattern::Rectilinear => 2,
        Pattern::AlignedRectilinear => 3,
        Pattern::ZigZag => 4,
        Pattern::CrossZag => 5,
        Pattern::LockedZag => 6,
        Pattern::Line => 7,
        Pattern::Grid => 8,
        Pattern::Triangles => 9,
        Pattern::TriHexagon => 10,
        Pattern::Cubic => 11,
        Pattern::AdaptiveCubic => 12,
        Pattern::QuarterCubic => 13,
        Pattern::SupportCubic => 14,
        Pattern::Lightning => 15,
        Pattern::Honeycomb => 16,
        Pattern::ThreeDHoneycomb => 17,
        Pattern::LateralHoneycomb => 18,
        Pattern::LateralLattice => 19,
        Pattern::CrossHatch => 20,
        Pattern::TpmsD => 21,
        Pattern::TpmsFk => 22,
        Pattern::Gyroid => 23,
        Pattern::Concentric => 24,
        Pattern::HilbertCurve => 25,
        Pattern::ArchimedeanChords => 26,
        Pattern::OctagramSpiral => 27,
    }
}

pub(super) const fn extrusion_role_rank(role: crate::ExtrusionRole) -> u8 {
    use crate::ExtrusionRole as Role;

    match role {
        Role::None => 0,
        Role::Perimeter => 1,
        Role::ExternalPerimeter => 2,
        Role::OverhangPerimeter => 3,
        Role::InternalInfill => 4,
        Role::SolidInfill => 5,
        Role::TopSolidInfill => 6,
        Role::BottomSurface => 7,
        Role::Ironing => 8,
        Role::BridgeInfill => 9,
        Role::InternalBridgeInfill => 10,
        Role::GapFill => 11,
        Role::Skirt => 12,
        Role::Brim => 13,
        Role::SupportMaterial => 14,
        Role::SupportMaterialInterface => 15,
        Role::SupportTransition => 16,
        Role::WipeTower => 17,
        Role::Custom => 18,
        Role::Mixed => 19,
    }
}
