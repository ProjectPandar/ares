mod config;
mod lift_type;
mod parsing;

pub use config::RetractLiftEnforce;
pub(crate) use config::{LayerChangeRetraction, ZHopLiftMode};
use lift_type::parse_z_hop_lift_config;
use parsing::{
    firmware_bool, first_bool, first_non_negative_f64, first_non_negative_f64_all_values,
    first_nullable_non_negative_f64_all_values, first_nullable_percent_fraction_all_values,
    first_nullable_retract_lift_enforce, first_percent_fraction_all_values, orca_serialized_bools,
    orca_serialized_nullable_bools, orca_serialized_nullable_numbers, retract_lift_enforce,
};

use super::SliceOptions;
use crate::SliceError;

const RETRACT_WHEN_CHANGING_LAYER: &str = "retract_when_changing_layer";
const FILAMENT_RETRACT_WHEN_CHANGING_LAYER: &str = "filament_retract_when_changing_layer";
const REDUCE_INFILL_RETRACTION: &str = "reduce_infill_retraction";
const RETRACTION_LENGTH: &str = "retraction_length";
const FILAMENT_RETRACTION_LENGTH: &str = "filament_retraction_length";
const RETRACTION_MINIMUM_TRAVEL: &str = "retraction_minimum_travel";
const FILAMENT_RETRACTION_MINIMUM_TRAVEL: &str = "filament_retraction_minimum_travel";
const RETRACTION_DISTANCES_WHEN_CUT: &str = "retraction_distances_when_cut";
const RETRACTION_DISTANCES_WHEN_EC: &str = "retraction_distances_when_ec";
const LONG_RETRACTIONS_WHEN_CUT: &str = "long_retractions_when_cut";
const LONG_RETRACTIONS_WHEN_EC: &str = "long_retractions_when_ec";
const RETRACTION_DISTANCE_WHEN_CUT_DEFAULT: f64 = 18.0;
const RETRACTION_DISTANCE_WHEN_CUT_MIN: f64 = 10.0;
const RETRACTION_DISTANCE_WHEN_CUT_MAX: f64 = 18.0;
const RETRACTION_DISTANCE_WHEN_EC_DEFAULT: f64 = 10.0;
const RETRACTION_DISTANCE_WHEN_EC_MIN: f64 = 0.0;
const RETRACTION_DISTANCE_WHEN_EC_MAX: f64 = 10.0;
const RETRACT_RESTART_EXTRA: &str = "retract_restart_extra";
const FILAMENT_RETRACT_RESTART_EXTRA: &str = "filament_retract_restart_extra";
const RETRACTION_SPEED: &str = "retraction_speed";
const DERETRACTION_SPEED: &str = "deretraction_speed";
const FILAMENT_RETRACTION_SPEED: &str = "filament_retraction_speed";
const FILAMENT_DERETRACTION_SPEED: &str = "filament_deretraction_speed";
const USE_FIRMWARE_RETRACTION: &str = "use_firmware_retraction";
const WIPE: &str = "wipe";
const FILAMENT_WIPE: &str = "filament_wipe";
const WIPE_DISTANCE: &str = "wipe_distance";
const FILAMENT_WIPE_DISTANCE: &str = "filament_wipe_distance";
const RETRACT_BEFORE_WIPE: &str = "retract_before_wipe";
const FILAMENT_RETRACT_BEFORE_WIPE: &str = "filament_retract_before_wipe";
const ROLE_BASED_WIPE_SPEED: &str = "role_based_wipe_speed";
const WIPE_SPEED: &str = "wipe_speed";
const Z_HOP: &str = "z_hop";
const FILAMENT_Z_HOP: &str = "filament_z_hop";
const RETRACT_LIFT_ABOVE: &str = "retract_lift_above";
const FILAMENT_RETRACT_LIFT_ABOVE: &str = "filament_retract_lift_above";
const RETRACT_LIFT_BELOW: &str = "retract_lift_below";
const FILAMENT_RETRACT_LIFT_BELOW: &str = "filament_retract_lift_below";
const RETRACT_LIFT_ENFORCE: &str = "retract_lift_enforce";
const FILAMENT_RETRACT_LIFT_ENFORCE: &str = "filament_retract_lift_enforce";

fn first_nullable_bool_all_values(
    key: &str,
    value: Option<&serde_json::Value>,
    default: bool,
) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(default);
    };
    Ok(orca_serialized_nullable_bools(key, Some(value), default)?[0].unwrap_or(default))
}

fn wipe_speed_feedrate(
    value: Option<&serde_json::Value>,
    travel_speed_mm_s: f64,
) -> Result<f64, SliceError> {
    let speed_mm_s = match value {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            WIPE_SPEED,
            value,
            travel_speed_mm_s,
        )?,
        None => travel_speed_mm_s * 0.8,
    };
    Ok(speed_mm_s * 60.0)
}

impl SliceOptions {
    pub(crate) fn start_gcode_retract_length(&self) -> Result<f64, SliceError> {
        first_non_negative_f64(RETRACTION_LENGTH, self.values().get(RETRACTION_LENGTH), 0.8)
    }

    pub(crate) fn retraction_distance_when_cut(&self) -> Result<f64, SliceError> {
        Ok(self.retraction_distances_when_cut()?[0])
    }

    pub(crate) fn retraction_distance_when_ec(&self) -> Result<f64, SliceError> {
        self.retraction_distances_when_ec()?[0].ok_or_else(|| {
            SliceError::InvalidInput(format!("{RETRACTION_DISTANCES_WHEN_EC} scalar must not be nil"))
        })
    }

    pub(crate) fn retraction_distances_when_cut(&self) -> Result<Vec<f64>, SliceError> {
        let Some(value) = self.values().get(RETRACTION_DISTANCES_WHEN_CUT) else {
            return Ok(vec![RETRACTION_DISTANCE_WHEN_CUT_DEFAULT]);
        };
        let values = crate::options::parsing::parse_numeric_vector(
            RETRACTION_DISTANCES_WHEN_CUT,
            value,
        )?;
        if values.iter().all(|value| {
            value.is_finite()
                && *value >= RETRACTION_DISTANCE_WHEN_CUT_MIN
                && *value <= RETRACTION_DISTANCE_WHEN_CUT_MAX
        }) {
            Ok(values)
        } else {
            Err(SliceError::InvalidInput(format!(
                "{RETRACTION_DISTANCES_WHEN_CUT} is out of range"
            )))
        }
    }

    pub(crate) fn retraction_distances_when_ec(&self) -> Result<Vec<Option<f64>>, SliceError> {
        let values = match self.values().get(RETRACTION_DISTANCES_WHEN_EC) {
            Some(value) => orca_serialized_nullable_numbers(RETRACTION_DISTANCES_WHEN_EC, value)?,
            None => vec![Some(RETRACTION_DISTANCE_WHEN_EC_DEFAULT)],
        };
        if values.iter().all(|value| {
            value.is_none_or(|value| {
                value.is_finite()
                    && (RETRACTION_DISTANCE_WHEN_EC_MIN..=RETRACTION_DISTANCE_WHEN_EC_MAX)
                        .contains(&value)
            })
        }) {
            Ok(values)
        } else {
            Err(SliceError::InvalidInput(format!(
                "{RETRACTION_DISTANCES_WHEN_EC} is out of range"
            )))
        }
    }

    pub(crate) fn long_retraction_when_cut(&self) -> Result<bool, SliceError> {
        Ok(self.long_retractions_when_cut()?[0])
    }

    pub(crate) fn long_retraction_when_ec(&self) -> Result<bool, SliceError> {
        Ok(self.long_retractions_when_ec()?[0].unwrap_or(true))
    }

    pub(crate) fn long_retractions_when_cut(&self) -> Result<Vec<bool>, SliceError> {
        orca_serialized_bools(
            LONG_RETRACTIONS_WHEN_CUT,
            self.values().get(LONG_RETRACTIONS_WHEN_CUT),
            false,
        )
    }

    pub(crate) fn long_retractions_when_ec(&self) -> Result<Vec<Option<bool>>, SliceError> {
        orca_serialized_nullable_bools(
            LONG_RETRACTIONS_WHEN_EC,
            self.values().get(LONG_RETRACTIONS_WHEN_EC),
            false,
        )
    }

    pub(crate) fn layer_change_retraction(&self) -> Result<LayerChangeRetraction, SliceError> {
        let enabled = first_bool(
            RETRACT_WHEN_CHANGING_LAYER,
            self.values().get(RETRACT_WHEN_CHANGING_LAYER),
            false,
        )?;
        let enabled = first_nullable_bool_all_values(
            FILAMENT_RETRACT_WHEN_CHANGING_LAYER,
            self.values().get(FILAMENT_RETRACT_WHEN_CHANGING_LAYER),
            enabled,
        )?;
        let reduce_infill_retraction = first_bool(
            REDUCE_INFILL_RETRACTION,
            self.values().get(REDUCE_INFILL_RETRACTION),
            false,
        )?;
        let length = first_non_negative_f64_all_values(
            FILAMENT_RETRACTION_LENGTH,
            self.values().get(FILAMENT_RETRACTION_LENGTH),
            first_non_negative_f64(RETRACTION_LENGTH, self.values().get(RETRACTION_LENGTH), 0.8)?,
        )?;
        let minimum_travel = first_non_negative_f64_all_values(
            FILAMENT_RETRACTION_MINIMUM_TRAVEL,
            self.values().get(FILAMENT_RETRACTION_MINIMUM_TRAVEL),
            first_non_negative_f64_all_values(
                RETRACTION_MINIMUM_TRAVEL,
                self.values().get(RETRACTION_MINIMUM_TRAVEL),
                2.0,
            )?,
        )?;
        let restart_extra = first_non_negative_f64_all_values(
            FILAMENT_RETRACT_RESTART_EXTRA,
            self.values().get(FILAMENT_RETRACT_RESTART_EXTRA),
            first_non_negative_f64(
                RETRACT_RESTART_EXTRA,
                self.values().get(RETRACT_RESTART_EXTRA),
                0.0,
            )?,
        )?;
        let retract_speed = first_non_negative_f64(
            FILAMENT_RETRACTION_SPEED,
            self.values().get(FILAMENT_RETRACTION_SPEED),
            first_non_negative_f64(RETRACTION_SPEED, self.values().get(RETRACTION_SPEED), 30.0)?,
        )?;
        let deretract_speed = first_non_negative_f64(
            FILAMENT_DERETRACTION_SPEED,
            self.values().get(FILAMENT_DERETRACTION_SPEED),
            first_non_negative_f64(
                DERETRACTION_SPEED,
                self.values().get(DERETRACTION_SPEED),
                0.0,
            )?,
        )?;
        let use_firmware = firmware_bool(
            USE_FIRMWARE_RETRACTION,
            self.values().get(USE_FIRMWARE_RETRACTION),
            false,
        )?;
        let wipe = first_bool(WIPE, self.values().get(WIPE), false)?;
        let wipe = first_nullable_bool_all_values(
            FILAMENT_WIPE,
            self.values().get(FILAMENT_WIPE),
            wipe,
        )?;
        let wipe_distance = first_nullable_non_negative_f64_all_values(
            FILAMENT_WIPE_DISTANCE,
            self.values().get(FILAMENT_WIPE_DISTANCE),
            first_non_negative_f64_all_values(
                WIPE_DISTANCE,
                self.values().get(WIPE_DISTANCE),
                1.0,
            )?,
        )?;
        let retract_before_wipe = first_nullable_percent_fraction_all_values(
            FILAMENT_RETRACT_BEFORE_WIPE,
            self.values().get(FILAMENT_RETRACT_BEFORE_WIPE),
            first_percent_fraction_all_values(
                RETRACT_BEFORE_WIPE,
                self.values().get(RETRACT_BEFORE_WIPE),
                100.0,
            )?,
        )?;
        let role_based_wipe_speed = first_bool(
            ROLE_BASED_WIPE_SPEED,
            self.values().get(ROLE_BASED_WIPE_SPEED),
            true,
        )?;
        let wipe_feedrate =
            wipe_speed_feedrate(self.values().get(WIPE_SPEED), self.speed_options()?.travel_speed_mm_s())?;
        let z_hop = first_non_negative_f64_all_values(
            FILAMENT_Z_HOP,
            self.values().get(FILAMENT_Z_HOP),
            first_non_negative_f64(Z_HOP, self.values().get(Z_HOP), 0.4)?,
        )?;
        let z_hop_lift = parse_z_hop_lift_config(self.values())?;
        let resolution = self.resolution()?;
        let lift_above = first_nullable_non_negative_f64_all_values(
            FILAMENT_RETRACT_LIFT_ABOVE,
            self.values().get(FILAMENT_RETRACT_LIFT_ABOVE),
            first_non_negative_f64(
                RETRACT_LIFT_ABOVE,
                self.values().get(RETRACT_LIFT_ABOVE),
                0.0,
            )?,
        )?;
        let lift_below = first_nullable_non_negative_f64_all_values(
            FILAMENT_RETRACT_LIFT_BELOW,
            self.values().get(FILAMENT_RETRACT_LIFT_BELOW),
            first_non_negative_f64(
                RETRACT_LIFT_BELOW,
                self.values().get(RETRACT_LIFT_BELOW),
                0.0,
            )?,
        )?;
        let lift_enforce = first_nullable_retract_lift_enforce(
            FILAMENT_RETRACT_LIFT_ENFORCE,
            self.values().get(FILAMENT_RETRACT_LIFT_ENFORCE),
            retract_lift_enforce(
                RETRACT_LIFT_ENFORCE,
                self.values().get(RETRACT_LIFT_ENFORCE),
            )?,
        )?;

        let unretract_speed = if deretract_speed == 0.0 {
            retract_speed
        } else {
            deretract_speed
        };
        Ok(LayerChangeRetraction {
            layer_change_enabled: enabled,
            reduce_infill_retraction,
            length,
            unretract_length: length + restart_extra,
            retract_feedrate: retract_speed * 60.0,
            unretract_feedrate: unretract_speed * 60.0,
            use_firmware,
            wipe,
            wipe_distance,
            retract_before_wipe,
            role_based_wipe_speed,
            wipe_feedrate,
            z_hop,
            z_hop_lift,
            resolution,
            lift_above,
            lift_below,
            lift_enforce,
            minimum_travel,
        })
    }
}
