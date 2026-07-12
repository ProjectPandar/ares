pub(crate) const AUTO_WIDTH_RATIO: f64 = 1.125;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ExtrusionWidthSpec {
    Absolute(f64),
    Percent(f64),
}

impl ExtrusionWidthSpec {
    pub(crate) const fn auto() -> Self {
        Self::Absolute(0.0)
    }

    pub(crate) const fn absolute(value: f64) -> Self {
        Self::Absolute(value)
    }

    pub(crate) const fn percent(value: f64) -> Self {
        Self::Percent(value)
    }

    pub(crate) fn resolve(self, nozzle_diameter: f64) -> f64 {
        match self {
            Self::Absolute(value) => value,
            Self::Percent(value) => value / 100.0 * nozzle_diameter,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RoleHardwareValues {
    pub(crate) nozzle_diameter: f64,
    pub(crate) filament_diameter: f64,
}

impl RoleHardwareValues {
    pub(crate) const fn new(nozzle_diameter: f64, filament_diameter: f64) -> Self {
        Self {
            nozzle_diameter,
            filament_diameter,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RoleExtrusionHardware {
    pub(crate) default: RoleHardwareValues,
    pub(crate) wall: RoleHardwareValues,
    pub(crate) sparse_infill: RoleHardwareValues,
    pub(crate) solid_infill: RoleHardwareValues,
    pub(crate) support: RoleHardwareValues,
    pub(crate) support_interface: RoleHardwareValues,
}

impl RoleExtrusionHardware {
    pub(crate) const fn first(nozzle_diameter: f64, filament_diameter: f64) -> Self {
        let default = RoleHardwareValues::new(nozzle_diameter, filament_diameter);
        Self::from_default(default)
    }

    pub(crate) const fn from_default(default: RoleHardwareValues) -> Self {
        Self {
            default,
            wall: default,
            sparse_infill: default,
            solid_infill: default,
            support: default,
            support_interface: default,
        }
    }

    pub(crate) const fn with_wall(self, wall: RoleHardwareValues) -> Self {
        Self { wall, ..self }
    }

    pub(crate) const fn with_sparse_infill(self, sparse_infill: RoleHardwareValues) -> Self {
        Self {
            sparse_infill,
            ..self
        }
    }

    pub(crate) const fn with_solid_infill(self, solid_infill: RoleHardwareValues) -> Self {
        Self {
            solid_infill,
            ..self
        }
    }

    pub(crate) const fn with_support(self, support: RoleHardwareValues) -> Self {
        Self { support, ..self }
    }

    pub(crate) const fn with_support_interface(
        self,
        support_interface: RoleHardwareValues,
    ) -> Self {
        Self {
            support_interface,
            ..self
        }
    }
}
