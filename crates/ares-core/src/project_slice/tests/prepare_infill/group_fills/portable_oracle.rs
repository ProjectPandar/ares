mod full_ksr;
mod render;
#[cfg(test)]
mod tests;
mod types;

use std::fmt::Write as _;

use sha2::{Digest, Sha256};

use render::{encode_layer_geometry, encode_layer_metadata, encode_layer_table};
pub(super) use types::{
    EncodedOracle, OracleFlow, OracleGroup, OracleLayer, OracleLockCounts, OracleParams,
    OracleRepresentative, OracleTotals,
};

pub(super) const PINNED_ORCA_COMMIT: &str = "8500fcdccaa10b5099ac20d252af3a7c560046f1";
pub(super) const INSTRUMENTATION_PATCH_SHA256: &str =
    "582e53cd1162f573fe4facbafb21e7f360431505e314726e7bb85e7f7221bc52";
pub(super) const PRE_METADATA_SHA256: &str =
    "a091ca0a63e45dc81712223571b1dfe888ab256bec2437ea564f386783f77900";
pub(super) const PRE_CANONICAL_GEOMETRY_SHA256: &str =
    "062fab2bbcb683df778ac024a8f6abed7960f3ebac3d55f13124617694d7e2af";
pub(super) const PRE_LAYER_TABLE_SHA256: &str =
    "ebd74a25609827e4affda26a21d9cd3b10dca08778f56f394b5170f74ecdf721";
pub(super) const O74_POST_METADATA_SHA256: &str =
    "cd4aa18a831dd4672e3e394944e496b8d349b5e21990672a7f14868cc2b3b387";
pub(super) const O74_POST_CANONICAL_GEOMETRY_SHA256: &str =
    "c149d65f5e5ddb89643b78314861ac2343707ddf76decc1e6aa2f88901331f6c";
pub(super) const O74_POST_LAYER_TABLE_SHA256: &str =
    "8d9845b22e38857dbb0840b2527286436a6b9c684c8662d925f8fd4873cef5b2";
pub(super) const LINUX_PRE_METADATA_SHA256: &str =
    "25a9ddd67028354ff44607a59c04a065ffa74a99b9f1a05bdc7a1adb9c15dce7";
pub(super) const LINUX_PRE_CANONICAL_GEOMETRY_SHA256: &str =
    "136cca449aebb9d155fd51552f51a7bb3b2f5acb42702bd84b2d2920e265d1dc";
pub(super) const LINUX_PRE_LAYER_TABLE_SHA256: &str =
    "f45a91b4f62dabae2f2320f936b8c903ee5d8e7d8db07fb9251418c82e832bf6";
pub(super) const LINUX_POST_METADATA_SHA256: &str =
    "36aecdaf4d3bfb8dadcaf63a0d0d39f3a12ad9b0b0e1aad0c5a9ceab19ef2eff";
pub(super) const LINUX_POST_CANONICAL_GEOMETRY_SHA256: &str =
    "13d36da11e01e99840b1cf058003ad18c26c29bd8d6bb0d33af23c1b2ce4534c";
pub(super) const LINUX_POST_LAYER_TABLE_SHA256: &str =
    "15dd3f792d2a9176630e30c2170487c872a9b94eb637fdb6eb6a2841667ece5a";

// This Linux instrumentation checksum is provenance, not a result oracle. The
// accepted hashes above replay the repository's fixed-MSVC bridge ordering.
pub(super) const NONPORTABLE_STABLE_RAW_SHA256: &str =
    "5ac8e44d9ab4f9c9e8954db375ee70fbfc5a38e16f66fd7038321bc7cdcd3124";
pub(super) const RAW_ORDER_VARIANT_LAYERS: [usize; 4] = [13, 18, 49, 259];

pub(super) const KSR_TOTALS: OracleTotals = OracleTotals {
    layers: 460,
    groups: 477,
    fill_expolygons: 1_882,
    fill_holes: 174,
    fill_paths: 2_056,
    fill_points: 107_540,
    no_overlap_expolygons: 2_547,
    nonempty_layers: 260,
    empty_layers: 200,
};
pub(super) const O74_POST_TOTALS: OracleTotals = OracleTotals {
    layers: 460,
    groups: 536,
    fill_expolygons: 2_218,
    fill_holes: 152,
    fill_paths: 2_370,
    fill_points: 110_610,
    no_overlap_expolygons: 2_928,
    nonempty_layers: 260,
    empty_layers: 200,
};
pub(super) const KSR_GROUP_HISTOGRAM: [(usize, usize); 7] = [
    (0, 200),
    (1, 105),
    (2, 107),
    (3, 40),
    (4, 5),
    (5, 2),
    (8, 1),
];
pub(super) const KSR_KIND_COUNTS: [(u8, usize); 6] =
    [(0, 31), (1, 1), (2, 11), (4, 252), (5, 160), (6, 22)];
pub(super) const KSR_PATTERN_COUNTS: [(u8, usize); 3] = [(0, 194), (1, 31), (20, 252)];
pub(super) const KSR_ROLE_COUNTS: [(u8, usize); 6] =
    [(4, 252), (5, 160), (6, 31), (7, 1), (9, 11), (10, 22)];
pub(super) const KSR_EXTRUDER_COUNTS: [(u32, usize); 1] = [(1, 477)];
pub(super) const KSR_PARAMS_BRIDGE_COUNTS: [(bool, usize); 2] = [(false, 444), (true, 33)];
pub(super) const KSR_FLOW_BRIDGE_COUNTS: [(bool, usize); 2] = [(false, 455), (true, 22)];
pub(super) const KSR_LOCK_COUNTS: OracleLockCounts = OracleLockCounts {
    skin_density: 0,
    skeleton_density: 0,
    skin_flow: 0,
    skeleton_flow: 0,
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

pub(super) fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
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
