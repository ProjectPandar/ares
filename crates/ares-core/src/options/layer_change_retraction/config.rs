#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub enum RetractLiftEnforce {
    #[default]
    #[serde(rename = "All Surfaces")]
    AllSurfaces,
    #[serde(rename = "Top Only")]
    TopOnly,
    #[serde(rename = "Bottom Only")]
    BottomOnly,
    #[serde(rename = "Top and Bottom")]
    TopAndBottom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ZHopLiftMode {
    Normal,
    Auto { radians: f64 },
    Slope { radians: f64 },
    Spiral { radians: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerChangeRetraction {
    pub(crate) layer_change_enabled: bool,
    pub(crate) reduce_infill_retraction: bool,
    pub(crate) length: f64,
    pub(crate) unretract_length: f64,
    pub(crate) retract_feedrate: f64,
    pub(crate) unretract_feedrate: f64,
    pub(crate) use_firmware: bool,
    pub(crate) wipe: bool,
    pub(crate) wipe_distance: f64,
    pub(crate) retract_before_wipe: f64,
    pub(crate) role_based_wipe_speed: bool,
    pub(crate) wipe_feedrate: f64,
    pub(crate) z_hop: f64,
    pub(crate) z_hop_lift: ZHopLiftMode,
    pub(crate) resolution: f64,
    pub(crate) lift_above: f64,
    pub(crate) lift_below: f64,
    pub(crate) lift_enforce: RetractLiftEnforce,
    pub(crate) minimum_travel: f64,
}

impl LayerChangeRetraction {
    pub(crate) fn is_enabled(self) -> bool {
        self.layer_change_enabled && self.length > 0.0
    }

    pub(crate) fn travel_retraction_enabled(self) -> bool {
        self.length > 0.0
    }

    pub(crate) fn z_hop_for_z(self, z: f64) -> f64 {
        if self.z_hop > 0.0
            && z >= self.lift_above
            && (self.lift_below == 0.0 || z <= self.lift_below)
        {
            self.z_hop
        } else {
            0.0
        }
    }

    pub(crate) fn z_hop_for_layer_change(
        self,
        z: f64,
        leaving_first_layer: bool,
        previous_non_gap_fill_role: Option<crate::PrintPathRole>,
    ) -> f64 {
        if !self
            .lift_enforce
            .allows(leaving_first_layer, previous_non_gap_fill_role)
        {
            return 0.0;
        }
        self.z_hop_for_z(z)
    }
}

impl RetractLiftEnforce {
    pub(crate) const fn allows(
        self,
        leaving_first_layer: bool,
        previous_non_gap_fill_role: Option<crate::PrintPathRole>,
    ) -> bool {
        match self {
            Self::AllSurfaces => true,
            Self::TopOnly => matches!(
                previous_non_gap_fill_role,
                Some(crate::PrintPathRole::TopSolidInfill)
            ),
            Self::BottomOnly => leaving_first_layer,
            Self::TopAndBottom => {
                leaving_first_layer
                    || matches!(
                        previous_non_gap_fill_role,
                        Some(crate::PrintPathRole::TopSolidInfill)
                    )
            }
        }
    }
}
