use crate::{SliceError, SliceOptions};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TreeSupportOptions {
    branch_distance_mm: f64,
    tip_diameter_mm: f64,
    branch_diameter_mm: f64,
    branch_angle_degrees: f64,
    branch_diameter_angle_degrees: f64,
    angle_slow_degrees: f64,
    wall_count: u32,
    auto_brim: bool,
    brim_width_mm: f64,
    branch_distance_organic_mm: f64,
    top_rate_percent: f64,
    branch_diameter_organic_mm: f64,
    branch_angle_organic_degrees: f64,
}

impl TreeSupportOptions {
    pub(crate) const fn branch_distance_mm(self) -> f64 {
        self.branch_distance_mm
    }

    pub(crate) const fn tip_diameter_mm(self) -> f64 {
        self.tip_diameter_mm
    }

    pub(crate) const fn branch_diameter_mm(self) -> f64 {
        self.branch_diameter_mm
    }

    pub(crate) const fn branch_angle_degrees(self) -> f64 {
        self.branch_angle_degrees
    }

    pub(crate) const fn branch_diameter_angle_degrees(self) -> f64 {
        self.branch_diameter_angle_degrees
    }

    pub(crate) const fn angle_slow_degrees(self) -> f64 {
        self.angle_slow_degrees
    }

    pub(crate) const fn wall_count(self) -> u32 {
        self.wall_count
    }

    pub(crate) const fn auto_brim(self) -> bool {
        self.auto_brim
    }

    pub(crate) const fn brim_width_mm(self) -> f64 {
        self.brim_width_mm
    }

    pub(crate) const fn branch_distance_organic_mm(self) -> f64 {
        self.branch_distance_organic_mm
    }

    pub(crate) const fn top_rate_percent(self) -> f64 {
        self.top_rate_percent
    }

    pub(crate) const fn branch_diameter_organic_mm(self) -> f64 {
        self.branch_diameter_organic_mm
    }

    pub(crate) const fn branch_angle_organic_degrees(self) -> f64 {
        self.branch_angle_organic_degrees
    }

    pub(crate) fn consume_runtime(self) {
        let _ = (
            self.branch_distance_mm(),
            self.tip_diameter_mm(),
            self.branch_diameter_mm(),
            self.branch_angle_degrees(),
            self.branch_diameter_angle_degrees(),
            self.angle_slow_degrees(),
            self.wall_count(),
            self.auto_brim(),
            self.brim_width_mm(),
            self.branch_distance_organic_mm(),
            self.top_rate_percent(),
            self.branch_diameter_organic_mm(),
            self.branch_angle_organic_degrees(),
        );
    }
}

fn parse_wall_count(options: &SliceOptions) -> Result<u32, SliceError> {
    let Some(value) = options.values.get("tree_support_wall_count") else {
        return Ok(0);
    };
    let value = match value {
        serde_json::Value::Number(number) => number.as_u64().and_then(|value| u32::try_from(value).ok()),
        serde_json::Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("tree_support_wall_count must be an integer".to_owned())
    })?;

    if value <= 2 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(
            "tree_support_wall_count is out of range".to_owned(),
        ))
    }
}

impl SliceOptions {
    pub(crate) fn tree_support_options(&self) -> Result<TreeSupportOptions, SliceError> {
        Ok(TreeSupportOptions {
            branch_distance_mm: self.range_f64("tree_support_branch_distance", 5.0, 1.0, 10.0)?,
            tip_diameter_mm: self.range_f64("tree_support_tip_diameter", 0.8, 0.1, 100.0)?,
            branch_diameter_mm: self.range_f64("tree_support_branch_diameter", 5.0, 1.0, 10.0)?,
            branch_angle_degrees: self.range_f64("tree_support_branch_angle", 40.0, 0.0, 60.0)?,
            branch_diameter_angle_degrees: self.range_f64(
                "tree_support_branch_diameter_angle",
                5.0,
                0.0,
                15.0,
            )?,
            angle_slow_degrees: self.range_f64("tree_support_angle_slow", 25.0, 10.0, 85.0)?,
            wall_count: parse_wall_count(self)?,
            auto_brim: self.bool_option("tree_support_auto_brim", true)?,
            brim_width_mm: self.range_f64("tree_support_brim_width", 3.0, 0.0, f64::INFINITY)?,
            branch_distance_organic_mm: self.range_f64(
                "tree_support_branch_distance_organic",
                1.0,
                1.0,
                10.0,
            )?,
            top_rate_percent: self.range_f64("tree_support_top_rate", 30.0, 5.0, 35.0)?,
            branch_diameter_organic_mm: self.range_f64(
                "tree_support_branch_diameter_organic",
                2.0,
                1.0,
                10.0,
            )?,
            branch_angle_organic_degrees: self.range_f64(
                "tree_support_branch_angle_organic",
                40.0,
                0.0,
                60.0,
            )?,
        })
    }
}
