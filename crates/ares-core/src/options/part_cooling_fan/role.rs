use super::PartCoolingFanRamp;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InternalBridgeFanSpeed {
    OverhangFallback,
    Fixed(u8),
}

impl InternalBridgeFanSpeed {
    pub(crate) const fn fallback() -> Self {
        Self::OverhangFallback
    }

    pub(crate) const fn new(speed: u8) -> Self {
        Self::Fixed(speed)
    }

    const fn role_speed(self, fallback_speed: Option<u8>) -> Option<u8> {
        match self {
            Self::OverhangFallback => fallback_speed,
            Self::Fixed(speed) => Some(speed),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RoleFanSpeed {
    Disabled,
    Fixed(u8),
}

impl RoleFanSpeed {
    pub(crate) const fn disabled() -> Self {
        Self::Disabled
    }

    pub(crate) const fn new(speed: u8) -> Self {
        Self::Fixed(speed)
    }

    const fn for_open_layer(self) -> Option<u8> {
        match self {
            Self::Disabled => None,
            Self::Fixed(speed) => Some(speed),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverhangFanThreshold {
    AllExternalPerimeters,
    OverlapGated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RoleFanControl {
    enabled: bool,
    overhang_speed: u8,
    internal_bridge_speed: InternalBridgeFanSpeed,
    support_interface_speed: RoleFanSpeed,
    ironing_speed: RoleFanSpeed,
    threshold: OverhangFanThreshold,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LayerRoleFanControl {
    bridge_speed: Option<u8>,
    internal_bridge_speed: Option<u8>,
    overhang_speed: Option<u8>,
    external_perimeter_speed: Option<u8>,
    support_interface_speed: Option<u8>,
    ironing_speed: Option<u8>,
}

impl RoleFanControl {
    pub(crate) const fn new(
        enabled: bool,
        overhang_speed: u8,
        internal_bridge_speed: InternalBridgeFanSpeed,
        threshold: OverhangFanThreshold,
    ) -> Self {
        Self {
            enabled,
            overhang_speed,
            internal_bridge_speed,
            support_interface_speed: RoleFanSpeed::disabled(),
            ironing_speed: RoleFanSpeed::disabled(),
            threshold,
        }
    }

    pub(crate) const fn with_support_interface_speed(mut self, speed: RoleFanSpeed) -> Self {
        self.support_interface_speed = speed;
        self
    }

    pub(crate) const fn with_ironing_speed(mut self, speed: RoleFanSpeed) -> Self {
        self.ironing_speed = speed;
        self
    }

    pub(crate) fn for_layer(
        self,
        ramp: PartCoolingFanRamp,
        layer_index: usize,
        baseline_speed: Option<u8>,
    ) -> LayerRoleFanControl {
        let layer_id = u32::try_from(layer_index).unwrap_or(u32::MAX);
        let role_layer_open = layer_id >= ramp.close_fan_first_layers();
        let support_interface_speed = role_layer_open
            .then_some(self.support_interface_speed)
            .and_then(RoleFanSpeed::for_open_layer);
        let ironing_speed = role_layer_open
            .then_some(self.ironing_speed)
            .and_then(RoleFanSpeed::for_open_layer);
        if !self.enabled {
            return LayerRoleFanControl::disabled()
                .with_support_interface_speed(support_interface_speed)
                .with_ironing_speed(ironing_speed);
        }
        let Some(overhang_speed) = ramp.role_speed_for_layer(layer_index, self.overhang_speed)
        else {
            return LayerRoleFanControl::disabled()
                .with_support_interface_speed(support_interface_speed)
                .with_ironing_speed(ironing_speed);
        };
        let role_speed = (overhang_speed > baseline_speed.unwrap_or(0)).then_some(overhang_speed);
        let internal_bridge_speed = self.internal_bridge_speed.role_speed(role_speed);
        let external_perimeter_speed = match self.threshold {
            OverhangFanThreshold::AllExternalPerimeters => role_speed,
            OverhangFanThreshold::OverlapGated => None,
        };
        LayerRoleFanControl {
            bridge_speed: role_speed,
            internal_bridge_speed,
            overhang_speed: role_speed,
            external_perimeter_speed,
            support_interface_speed,
            ironing_speed,
        }
    }
}

impl LayerRoleFanControl {
    pub(crate) const fn disabled() -> Self {
        Self {
            bridge_speed: None,
            internal_bridge_speed: None,
            overhang_speed: None,
            external_perimeter_speed: None,
            support_interface_speed: None,
            ironing_speed: None,
        }
    }

    const fn with_support_interface_speed(mut self, speed: Option<u8>) -> Self {
        self.support_interface_speed = speed;
        self
    }

    const fn with_ironing_speed(mut self, speed: Option<u8>) -> Self {
        self.ironing_speed = speed;
        self
    }

    pub(crate) const fn speed_for_role(self, role: crate::PrintPathRole) -> Option<u8> {
        match role {
            crate::PrintPathRole::Bridge => self.bridge_speed,
            crate::PrintPathRole::InternalBridge => self.internal_bridge_speed,
            crate::PrintPathRole::OverhangPerimeter => self.overhang_speed,
            crate::PrintPathRole::ExternalPerimeter => self.external_perimeter_speed,
            crate::PrintPathRole::SupportMaterialInterface => self.support_interface_speed,
            crate::PrintPathRole::Ironing => self.ironing_speed,
            _ => None,
        }
    }
}
