#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum InternalBridgeFanSpeed {
    #[default]
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

    pub(crate) const fn role_speed(self, fallback_speed: Option<u8>) -> Option<u8> {
        match self {
            Self::OverhangFallback => fallback_speed,
            Self::Fixed(speed) => Some(speed),
        }
    }
}
