use crate::{PrintPathRole, SliceError};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SmallAreaInfillFlowCompensation {
    model: Option<PchipModel>,
    bottom_supported: bool,
    internal_supported: bool,
    top_supported: bool,
}

impl SmallAreaInfillFlowCompensation {
    pub(crate) const fn disabled() -> Self {
        Self {
            model: None,
            bottom_supported: false,
            internal_supported: false,
            top_supported: false,
        }
    }

    pub(crate) const fn default_model_entries() -> &'static [&'static str] {
        &[
            "0,0",
            "\n0.2,0.4444",
            "\n0.4,0.6145",
            "\n0.6,0.7059",
            "\n0.8,0.7619",
            "\n1.5,0.8571",
            "\n2,0.8889",
            "\n3,0.9231",
            "\n5,0.9520",
            "\n10,1",
        ]
    }

    pub(crate) fn parse(
        entries: Vec<String>,
        bottom_supported: bool,
        internal_supported: bool,
        top_supported: bool,
    ) -> Result<Self, SliceError> {
        Ok(Self {
            model: Some(PchipModel::new(parse_points(entries)?)),
            bottom_supported,
            internal_supported,
            top_supported,
        })
    }

    pub(crate) fn multiplier(
        &self,
        role: PrintPathRole,
        is_first_layer: bool,
        line_length_mm: f64,
    ) -> f64 {
        let Some(model) = &self.model else {
            return 1.0;
        };
        if !matches!(
            role,
            PrintPathRole::SolidInfill
                | PrintPathRole::TopSolidInfill
                | PrintPathRole::BottomSurface
        ) {
            return 1.0;
        }
        if !((is_first_layer && self.bottom_supported)
            || (role == PrintPathRole::SolidInfill && self.internal_supported)
            || (role == PrintPathRole::TopSolidInfill && self.top_supported))
        {
            return 1.0;
        }
        model.multiplier(line_length_mm)
    }

    pub(crate) fn multiplier_for_feature(&self, feature: &str, line_length_mm: f64) -> f64 {
        let Some(model) = &self.model else {
            return 1.0;
        };
        if matches!(
            feature,
            "Internal solid infill" | "Top surface" | "Bottom surface"
        ) {
            model.multiplier(line_length_mm)
        } else {
            1.0
        }
    }
}

impl Default for SmallAreaInfillFlowCompensation {
    fn default() -> Self {
        Self::disabled()
    }
}

fn parse_points(entries: Vec<String>) -> Result<Vec<(f64, f64)>, SliceError> {
    let mut points = Vec::new();
    for entry in entries {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        let mut fields = entry.split(',').map(str::trim);
        let Some(length) = fields.next() else {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model entries must be length,factor"
                    .to_owned(),
            ));
        };
        let Some(factor) = fields.next() else {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model entries must be length,factor"
                    .to_owned(),
            ));
        };
        if fields.next().is_some() {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model entries must be length,factor"
                    .to_owned(),
            ));
        }
        let length = length.parse::<f64>().map_err(|_| {
            SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model contains invalid length".to_owned(),
            )
        })?;
        let factor = factor.parse::<f64>().map_err(|_| {
            SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model contains invalid factor".to_owned(),
            )
        })?;
        if !length.is_finite() || !factor.is_finite() {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model values must be finite".to_owned(),
            ));
        }
        points.push((length, factor));
    }
    validate_points(&points)?;
    Ok(points)
}

fn validate_points(points: &[(f64, f64)]) -> Result<(), SliceError> {
    if points.len() < 2 {
        return Err(SliceError::InvalidInput(
            "small_area_infill_flow_compensation_model needs at least two points".to_owned(),
        ));
    }
    if points[0].0 != 0.0 {
        return Err(SliceError::InvalidInput(
            "small_area_infill_flow_compensation_model first small-area extrusion length must be 0"
                .to_owned(),
        ));
    }
    for window in points.windows(2) {
        if window[1].0 == 0.0 {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model only allows the first small-area extrusion length to be 0"
                    .to_owned(),
            ));
        }
        if window[1].0 <= window[0].0 {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model lengths must increase".to_owned(),
            ));
        }
        if window[1].1 <= window[0].1 {
            return Err(SliceError::InvalidInput(
                "small_area_infill_flow_compensation_model flow factors must increase".to_owned(),
            ));
        }
    }
    if points.last().map(|point| point.1) != Some(1.0) {
        return Err(SliceError::InvalidInput(
            "small_area_infill_flow_compensation_model final flow factor must be 1.0".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct PchipModel {
    x: Vec<f64>,
    y: Vec<f64>,
    h: Vec<f64>,
    d: Vec<f64>,
}

impl PchipModel {
    fn new(points: Vec<(f64, f64)>) -> Self {
        let (x, y): (Vec<_>, Vec<_>) = points.into_iter().unzip();
        let mut h = Vec::with_capacity(x.len() - 1);
        let mut delta = Vec::with_capacity(x.len() - 1);
        for i in 0..x.len() - 1 {
            h.push(x[i + 1] - x[i]);
            delta.push((y[i + 1] - y[i]) / h[i]);
        }
        let mut d = vec![0.0; x.len()];
        d[0] = delta[0];
        d[x.len() - 1] = delta[delta.len() - 1];
        for i in 1..x.len() - 1 {
            if delta[i - 1] * delta[i] > 0.0 {
                let w1 = 2.0 * h[i] + h[i - 1];
                let w2 = h[i] + 2.0 * h[i - 1];
                d[i] = (w1 + w2) / (w1 / delta[i - 1] + w2 / delta[i]);
            }
        }
        Self { x, y, h, d }
    }

    fn multiplier(&self, line_length_mm: f64) -> f64 {
        if line_length_mm == 0.0 || line_length_mm > self.x[self.x.len() - 1] {
            return 1.0;
        }
        if line_length_mm <= self.x[0] {
            return self.y[0];
        }
        if line_length_mm >= self.x[self.x.len() - 1] {
            return self.y[self.y.len() - 1];
        }
        let i = self.x.partition_point(|x| *x < line_length_mm) - 1;
        let h = self.h[i];
        let t = (line_length_mm - self.x[i]) / h;
        let t2 = t * t;
        let t3 = t2 * t;
        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + t;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;
        h00 * self.y[i] + h10 * h * self.d[i] + h01 * self.y[i + 1] + h11 * h * self.d[i + 1]
    }
}
