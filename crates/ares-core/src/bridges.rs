use std::collections::BTreeMap;

use serde_json::Value;

use crate::{SliceError, options::parsing::parse_positive_numeric_or_percent_over_base};

const DEFAULT_BRIDGE_FLOW: f64 = 1.0;
const DEFAULT_INTERNAL_BRIDGE_FLOW: f64 = 1.0;
const DEFAULT_BRIDGE_SPEED: f64 = 25.0;
const DEFAULT_INTERNAL_BRIDGE_SPEED_PERCENT: f64 = 150.0;
const DEFAULT_BRIDGE_NO_SUPPORT: bool = false;
const DEFAULT_THICK_BRIDGES: bool = false;
const DEFAULT_THICK_INTERNAL_BRIDGES: bool = true;
const DEFAULT_COUNTERBORE_HOLE_BRIDGING: CounterboreHoleBridging = CounterboreHoleBridging::None;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExtraBridgeLayer {
    Disabled,
    ExternalBridgeOnly,
    InternalBridgeOnly,
    ApplyToAll,
}

impl ExtraBridgeLayer {
    pub(crate) const fn applies_to_external_bridge(self) -> bool {
        matches!(self, Self::ExternalBridgeOnly | Self::ApplyToAll)
    }

    #[cfg(test)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::ExternalBridgeOnly => "external_bridge_only",
            Self::InternalBridgeOnly => "internal_bridge_only",
            Self::ApplyToAll => "apply_to_all",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CounterboreHoleBridging {
    None,
    PartiallyBridge,
    SacrificialLayer,
}

impl CounterboreHoleBridging {
    pub(crate) const fn preserves_bridge_surfaces_for_solid_detection(self) -> bool {
        matches!(self, Self::SacrificialLayer)
    }

    #[cfg(test)]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::PartiallyBridge => "partiallybridge",
            Self::SacrificialLayer => "sacrificiallayer",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BridgeLayerPolicy {
    bridge_no_support: bool,
    extra_bridge_layer: ExtraBridgeLayer,
    counterbore_hole_bridging: CounterboreHoleBridging,
}

impl BridgeLayerPolicy {
    pub(crate) const fn new(
        bridge_no_support: bool,
        extra_bridge_layer: ExtraBridgeLayer,
        counterbore_hole_bridging: CounterboreHoleBridging,
    ) -> Self {
        Self {
            bridge_no_support,
            extra_bridge_layer,
            counterbore_hole_bridging,
        }
    }

    pub(crate) const fn bridge_no_support(self) -> bool {
        self.bridge_no_support
    }

    pub(crate) const fn extra_bridge_layer(self) -> ExtraBridgeLayer {
        self.extra_bridge_layer
    }

    pub(crate) const fn unsupported_bottom_bridge_enabled(self) -> bool {
        self.bridge_no_support
            && !self
                .counterbore_hole_bridging
                .preserves_bridge_surfaces_for_solid_detection()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BridgeOptions {
    bridge_flow: f64,
    internal_bridge_flow: f64,
    bridge_speed_mm_s: f64,
    internal_bridge_speed_mm_s: f64,
    bridge_no_support: bool,
    thick_bridges: bool,
    thick_internal_bridges: bool,
    extra_bridge_layer: ExtraBridgeLayer,
    counterbore_hole_bridging: CounterboreHoleBridging,
}

impl BridgeOptions {
    pub const fn bridge_flow(&self) -> f64 {
        self.bridge_flow
    }

    pub const fn internal_bridge_flow(&self) -> f64 {
        self.internal_bridge_flow
    }

    pub const fn bridge_speed_mm_s(&self) -> f64 {
        self.bridge_speed_mm_s
    }

    pub const fn internal_bridge_speed_mm_s(&self) -> f64 {
        self.internal_bridge_speed_mm_s
    }

    pub const fn bridge_no_support(&self) -> bool {
        self.bridge_no_support
    }

    pub const fn thick_bridges(&self) -> bool {
        self.thick_bridges
    }

    pub const fn thick_internal_bridges(&self) -> bool {
        self.thick_internal_bridges
    }

    pub(crate) const fn extra_bridge_layer(&self) -> ExtraBridgeLayer {
        self.extra_bridge_layer
    }

    pub(crate) const fn counterbore_hole_bridging(&self) -> CounterboreHoleBridging {
        self.counterbore_hole_bridging
    }

    #[cfg(test)]
    pub(crate) const fn extra_bridge_layer_for_tests(&self) -> &'static str {
        self.extra_bridge_layer.as_str()
    }

    #[cfg(test)]
    pub(crate) const fn counterbore_hole_bridging_for_tests(&self) -> &'static str {
        self.counterbore_hole_bridging.as_str()
    }
}

pub(crate) fn parse_bridge_options(
    values: &BTreeMap<String, Value>,
) -> Result<BridgeOptions, SliceError> {
    let bridge_speed_mm_s = crate::options::parsing::parse_positive_number_or_string(
        "bridge_speed",
        values.get("bridge_speed"),
        DEFAULT_BRIDGE_SPEED,
    )?;
    Ok(BridgeOptions {
        bridge_flow: flow_multiplier(values, "bridge_flow", DEFAULT_BRIDGE_FLOW)?,
        internal_bridge_flow: flow_multiplier(
            values,
            "internal_bridge_flow",
            DEFAULT_INTERNAL_BRIDGE_FLOW,
        )?,
        bridge_speed_mm_s,
        internal_bridge_speed_mm_s: bridge_speed(values, bridge_speed_mm_s)?,
        bridge_no_support: boolean(values, "bridge_no_support", DEFAULT_BRIDGE_NO_SUPPORT)?,
        thick_bridges: boolean(values, "thick_bridges", DEFAULT_THICK_BRIDGES)?,
        thick_internal_bridges: boolean(
            values,
            "thick_internal_bridges",
            DEFAULT_THICK_INTERNAL_BRIDGES,
        )?,
        extra_bridge_layer: parse_extra_bridge_layer(values.get("enable_extra_bridge_layer"))?,
        counterbore_hole_bridging: parse_counterbore_hole_bridging(
            values.get("counterbore_hole_bridging"),
        )?,
    })
}

fn parse_counterbore_hole_bridging(
    value: Option<&Value>,
) -> Result<CounterboreHoleBridging, SliceError> {
    let Some(value) = value else {
        return Ok(DEFAULT_COUNTERBORE_HOLE_BRIDGING);
    };
    match value.as_str() {
        Some("none") => Ok(CounterboreHoleBridging::None),
        Some("partiallybridge") => Ok(CounterboreHoleBridging::PartiallyBridge),
        Some("sacrificiallayer") => Ok(CounterboreHoleBridging::SacrificialLayer),
        Some(_) => Err(SliceError::InvalidInput(
            "counterbore_hole_bridging has unknown enum value".to_owned(),
        )),
        None => Err(SliceError::InvalidInput(
            "counterbore_hole_bridging must be a string".to_owned(),
        )),
    }
}

fn parse_extra_bridge_layer(value: Option<&Value>) -> Result<ExtraBridgeLayer, SliceError> {
    let Some(value) = value else {
        return Ok(ExtraBridgeLayer::Disabled);
    };
    match value.as_str() {
        Some("disabled") => Ok(ExtraBridgeLayer::Disabled),
        Some("external_bridge_only") => Ok(ExtraBridgeLayer::ExternalBridgeOnly),
        Some("internal_bridge_only") => Ok(ExtraBridgeLayer::InternalBridgeOnly),
        Some("apply_to_all") => Ok(ExtraBridgeLayer::ApplyToAll),
        Some(_) => Err(SliceError::InvalidInput(
            "enable_extra_bridge_layer has unknown enum value".to_owned(),
        )),
        None => Err(SliceError::InvalidInput(
            "enable_extra_bridge_layer must be a string".to_owned(),
        )),
    }
}

fn bridge_speed(
    values: &BTreeMap<String, Value>,
    bridge_speed_mm_s: f64,
) -> Result<f64, SliceError> {
    let Some(value) = values.get("internal_bridge_speed") else {
        return Ok(DEFAULT_INTERNAL_BRIDGE_SPEED_PERCENT / 100.0 * bridge_speed_mm_s);
    };
    parse_positive_numeric_or_percent_over_base("internal_bridge_speed", value, bridge_speed_mm_s)
}

fn flow_multiplier(
    values: &BTreeMap<String, Value>,
    key: &str,
    default: f64,
) -> Result<f64, SliceError> {
    let value =
        crate::options::parsing::parse_positive_number_or_string(key, values.get(key), default)?;
    if value <= 2.0 {
        Ok(value)
    } else {
        Err(SliceError::InvalidInput(format!("{key} is out of range")))
    }
}

fn boolean(values: &BTreeMap<String, Value>, key: &str, default: bool) -> Result<bool, SliceError> {
    let Some(value) = values.get(key) else {
        return Ok(default);
    };
    value
        .as_bool()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must be a boolean")))
}
