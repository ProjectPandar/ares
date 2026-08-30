use serde_json::Value;

use super::fuzzy_skin_noise::{fuzzify_closed_polyline, ripple_closed_polyline};
use crate::{
    Point2, ProcessFuzzySkinType, ProcessNoiseType, RegionOptions, SliceError,
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
pub(super) enum FuzzySkinNoiseType {
    Classic,
    Perlin,
    Billow,
    RidgedMulti,
    Voronoi,
    Ripple,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FuzzySkinConfig {
    kind: FuzzySkinKind,
    noise_type: FuzzySkinNoiseType,
    pub(super) thickness_mm: f64,
    pub(super) point_distance_mm: f64,
    pub(super) scale_mm: f64,
    pub(super) octaves: usize,
    pub(super) persistence: f64,
    pub(super) ripples_per_layer: usize,
    pub(super) ripple_offset_percent: f64,
    pub(super) layers_between_ripple_offset: usize,
    first_layer: bool,
}

impl FuzzySkinConfig {
    pub(crate) const fn disabled() -> Self {
        Self {
            kind: FuzzySkinKind::Disabled,
            noise_type: FuzzySkinNoiseType::Classic,
            thickness_mm: 0.2,
            point_distance_mm: 0.3,
            scale_mm: 1.0,
            octaves: 4,
            persistence: 0.5,
            ripples_per_layer: 15,
            ripple_offset_percent: 50.0,
            layers_between_ripple_offset: 1,
            first_layer: false,
        }
    }

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

    pub(crate) fn parse(
        values: &std::collections::BTreeMap<String, Value>,
        first_layer: bool,
    ) -> Result<Self, SliceError> {
        Ok(Self {
            kind: parse_kind(values.get("fuzzy_skin"))?,
            noise_type: parse_noise_type(values.get("fuzzy_skin_noise_type"))?,
            thickness_mm: crate::options::parsing::parse_range_f64(
                "fuzzy_skin_thickness",
                values.get("fuzzy_skin_thickness"),
                0.2,
                0.0,
                2.0,
            )?,
            point_distance_mm: crate::options::parsing::parse_range_f64(
                "fuzzy_skin_point_distance",
                values.get("fuzzy_skin_point_distance"),
                0.3,
                0.0,
                5.0,
            )?,
            scale_mm: crate::options::parsing::parse_range_f64(
                "fuzzy_skin_scale",
                values.get("fuzzy_skin_scale"),
                1.0,
                0.1,
                500.0,
            )?,
            octaves: parse_range_usize(
                "fuzzy_skin_octaves",
                values.get("fuzzy_skin_octaves"),
                4,
                1,
                10,
            )?,
            persistence: crate::options::parsing::parse_range_f64(
                "fuzzy_skin_persistence",
                values.get("fuzzy_skin_persistence"),
                0.5,
                0.01,
                1.0,
            )?,
            ripples_per_layer: parse_positive_usize(
                "fuzzy_skin_ripples_per_layer",
                values.get("fuzzy_skin_ripples_per_layer"),
                15,
            )?,
            ripple_offset_percent: parse_percent(
                "fuzzy_skin_ripple_offset",
                values.get("fuzzy_skin_ripple_offset"),
                50.0,
            )?,
            layers_between_ripple_offset: parse_positive_usize(
                "fuzzy_skin_layers_between_ripple_offset",
                values.get("fuzzy_skin_layers_between_ripple_offset"),
                1,
            )?,
            first_layer,
        })
    }

    pub(crate) fn fuzzified_scaled_points(
        self,
        points: Vec<Point>,
        layer_id: usize,
        print_z: f64,
        scale: CoordinateScale,
    ) -> Vec<Point> {
        if self.noise_type == FuzzySkinNoiseType::Ripple {
            return super::fuzzy_skin_noise::ripple_scaled_closed_polyline(
                &points, self, layer_id, scale,
            );
        }
        self.fuzzified_points(
            points
                .iter()
                .map(|point| Point2::new(scale.unscale(point.x()), scale.unscale(point.y())))
                .collect(),
            layer_id,
            print_z,
        )
        .into_iter()
        .map(|point| {
            Point::new(
                scale
                    .checked_scale(point.x())
                    .expect("validated fuzzy point stays in the coordinate range"),
                scale
                    .checked_scale(point.y())
                    .expect("validated fuzzy point stays in the coordinate range"),
            )
        })
        .collect()
    }

    pub(crate) fn external_points(
        self,
        points: Vec<Point2>,
        layer_id: usize,
        print_z: f64,
    ) -> Vec<Point2> {
        if self.fuzzifies_external(layer_id) {
            self.fuzzified_points(points, layer_id, print_z)
        } else {
            points
        }
    }

    pub(crate) fn internal_wall_points(
        self,
        points: Vec<Point2>,
        layer_id: usize,
        print_z: f64,
    ) -> Vec<Point2> {
        if self.fuzzifies_internal_wall(layer_id) {
            self.fuzzified_points(points, layer_id, print_z)
        } else {
            points
        }
    }

    pub(crate) fn fuzzified_points(
        self,
        points: Vec<Point2>,
        layer_id: usize,
        print_z: f64,
    ) -> Vec<Point2> {
        match self.noise_type {
            FuzzySkinNoiseType::Ripple => ripple_closed_polyline(&points, self, layer_id),
            noise_type => fuzzify_closed_polyline(&points, self, layer_id, print_z, noise_type),
        }
    }

    fn fuzzifies_external(self, layer_id: usize) -> bool {
        self.should_fuzzify(layer_id, 0, true)
    }

    fn fuzzifies_internal_wall(self, layer_id: usize) -> bool {
        self.should_fuzzify(layer_id, 1, true)
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

fn parse_noise_type(value: Option<&Value>) -> Result<FuzzySkinNoiseType, SliceError> {
    let Some(value) = value else {
        return Ok(FuzzySkinNoiseType::Classic);
    };
    let Some(text) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "fuzzy_skin_noise_type must be a string".to_owned(),
        ));
    };
    match text {
        "classic" => Ok(FuzzySkinNoiseType::Classic),
        "perlin" => Ok(FuzzySkinNoiseType::Perlin),
        "billow" => Ok(FuzzySkinNoiseType::Billow),
        "ridgedmulti" => Ok(FuzzySkinNoiseType::RidgedMulti),
        "voronoi" => Ok(FuzzySkinNoiseType::Voronoi),
        "ripple" => Ok(FuzzySkinNoiseType::Ripple),
        _ => Err(SliceError::InvalidInput(
            "fuzzy_skin_noise_type has unknown enum value".to_owned(),
        )),
    }
}

fn parse_range_usize(
    key: &str,
    value: Option<&Value>,
    default: usize,
    min: usize,
    max: usize,
) -> Result<usize, SliceError> {
    let parsed = parse_positive_usize(key, value, default)?;
    if (min..=max).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}

fn parse_positive_usize(
    key: &str,
    value: Option<&Value>,
    default: usize,
) -> Result<usize, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a positive integer")))?;
    usize::try_from(parsed)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a positive integer")))
}

fn parse_percent(key: &str, value: Option<&Value>, default: f64) -> Result<f64, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    let parsed = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => {
            let text = text.trim();
            text.strip_suffix('%').unwrap_or(text).trim().parse().ok()
        }
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a percent")))?;
    if parsed.is_finite() && (0.0..=100.0).contains(&parsed) {
        Ok(parsed)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}

fn parse_kind(value: Option<&Value>) -> Result<FuzzySkinKind, SliceError> {
    let Some(value) = value else {
        return Ok(FuzzySkinKind::Disabled);
    };
    let Some(text) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "fuzzy_skin must be a string".to_owned(),
        ));
    };
    match text {
        "none" => Ok(FuzzySkinKind::None),
        "external" => Ok(FuzzySkinKind::External),
        "hole" => Ok(FuzzySkinKind::Hole),
        "all" => Ok(FuzzySkinKind::All),
        "allwalls" => Ok(FuzzySkinKind::AllWalls),
        "disabled_fuzzy" => Ok(FuzzySkinKind::Disabled),
        _ => Err(SliceError::InvalidInput(
            "fuzzy_skin has unknown enum value".to_owned(),
        )),
    }
}
