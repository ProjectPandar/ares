use crate::{
    ProcessFuzzySkinType, ProcessNoiseType, RegionOptions,
    geometry::{CoordinateScale, Point},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FuzzySkinKind {
    None,
    External,
    Hole,
    All,
    AllWalls,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FuzzySkinNoiseType {
    Classic,
    Perlin,
    Billow,
    RidgedMulti,
    Voronoi,
    Ripple,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FuzzySkinConfig {
    pub(crate) kind: FuzzySkinKind,
    pub(crate) noise_type: FuzzySkinNoiseType,
    pub(crate) thickness_mm: f64,
    pub(crate) point_distance_mm: f64,
    pub(super) scale_mm: f64,
    pub(super) octaves: usize,
    pub(super) persistence: f64,
    pub(crate) ripples_per_layer: usize,
    pub(crate) ripple_offset_percent: f64,
    pub(crate) layers_between_ripple_offset: usize,
    pub(crate) first_layer: bool,
}

/// Layer identity and Z inputs to fuzzy skin (`Layer.hpp::Layer`,
/// `PerimeterGenerator.cpp::fuzzy_skin`).
pub(crate) struct FuzzySkinLayer {
    pub(crate) layer_id: usize,
    pub(crate) print_z: f64,
}

impl FuzzySkinConfig {
    pub(crate) fn from_region(region: &RegionOptions) -> Self {
        Self {
            kind: match region.fuzzy_skin {
                ProcessFuzzySkinType::None => FuzzySkinKind::None,
                ProcessFuzzySkinType::External => FuzzySkinKind::External,
                ProcessFuzzySkinType::Hole => FuzzySkinKind::Hole,
                ProcessFuzzySkinType::All => FuzzySkinKind::All,
                ProcessFuzzySkinType::AllWalls => FuzzySkinKind::AllWalls,
                ProcessFuzzySkinType::Disabled => FuzzySkinKind::Disabled,
            },
            noise_type: match region.fuzzy_skin_noise_type {
                ProcessNoiseType::Classic => FuzzySkinNoiseType::Classic,
                ProcessNoiseType::Perlin => FuzzySkinNoiseType::Perlin,
                ProcessNoiseType::Billow => FuzzySkinNoiseType::Billow,
                ProcessNoiseType::RidgedMulti => FuzzySkinNoiseType::RidgedMulti,
                ProcessNoiseType::Voronoi => FuzzySkinNoiseType::Voronoi,
                ProcessNoiseType::Ripple => FuzzySkinNoiseType::Ripple,
            },
            thickness_mm: region.fuzzy_skin_thickness.0,
            point_distance_mm: region.fuzzy_skin_point_distance.0,
            scale_mm: region.fuzzy_skin_scale.0,
            octaves: usize::try_from(region.fuzzy_skin_octaves.0)
                .expect("normalized fuzzy_skin_octaves is positive"),
            persistence: region.fuzzy_skin_persistence.0,
            ripples_per_layer: usize::try_from(region.fuzzy_skin_ripples_per_layer.0)
                .expect("normalized fuzzy_skin_ripples_per_layer is positive"),
            ripple_offset_percent: region.fuzzy_skin_ripple_offset.0,
            layers_between_ripple_offset: usize::try_from(
                region.fuzzy_skin_layers_between_ripple_offset.0,
            )
            .expect("normalized fuzzy_skin_layers_between_ripple_offset is positive"),
            first_layer: region.fuzzy_skin_first_layer.0,
        }
    }

    pub(crate) fn fuzzified_scaled_points(
        self,
        (points, closed): (&[Point], bool),
        FuzzySkinLayer { layer_id, print_z }: FuzzySkinLayer,
        scale: CoordinateScale,
        context: &mut crate::SlicerContext,
    ) -> Vec<Point> {
        if self.noise_type == FuzzySkinNoiseType::Ripple {
            return super::fuzzy_skin_noise::ripple_scaled_closed_polyline(
                points, self, layer_id, scale,
            );
        }
        super::fuzzy_skin_seeded::fuzzify((points, closed), self, print_z, scale, context.rng())
    }

    pub(crate) fn should_fuzzify(
        self,
        layer_id: usize,
        loop_index: usize,
        is_contour: bool,
    ) -> bool {
        if matches!(self.kind, FuzzySkinKind::None | FuzzySkinKind::Disabled)
            || !self.fuzzy_skin_effect_enabled(layer_id)
        {
            return false;
        }
        let fuzzify_contours = (loop_index == 0 && self.kind != FuzzySkinKind::Hole)
            || self.kind == FuzzySkinKind::AllWalls;
        let fuzzify_holes = matches!(
            self.kind,
            FuzzySkinKind::Hole | FuzzySkinKind::All | FuzzySkinKind::AllWalls
        ) && (loop_index == 0 || self.kind == FuzzySkinKind::AllWalls);
        if is_contour {
            fuzzify_contours
        } else {
            fuzzify_holes
        }
    }

    fn fuzzy_skin_effect_enabled(self, layer_id: usize) -> bool {
        if !self.first_layer && layer_id == 0 {
            return false;
        }
        self.point_distance_mm >= 0.01 && self.thickness_mm >= 0.001
    }
}
