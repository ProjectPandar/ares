pub(super) struct MoveComments {
    pub(super) speed: Option<&'static str>,
    pub(super) acceleration: Option<&'static str>,
    pub(super) jerk: Option<&'static str>,
    pub(super) z_travel: Option<&'static str>,
    pub(super) z_lift: Option<&'static str>,
    pub(super) z_restore: Option<&'static str>,
    pub(super) travel: Option<&'static str>,
    pub(super) extrude: Option<&'static str>,
    pub(super) retract: Option<&'static str>,
    pub(super) unretract: Option<&'static str>,
}

impl MoveComments {
    pub(super) fn new(enabled: bool) -> Self {
        Self {
            speed: enabled.then_some("set speed"),
            acceleration: enabled.then_some("adjust acceleration"),
            jerk: enabled.then_some("adjust jerk"),
            z_travel: enabled.then_some("move to layer Z"),
            z_lift: enabled.then_some("lift Z"),
            z_restore: enabled.then_some("restore layer Z"),
            travel: enabled.then_some("travel"),
            extrude: enabled.then_some("extrude"),
            retract: enabled.then_some("retract"),
            unretract: enabled.then_some("unretract"),
        }
    }
}

pub(super) fn max_print_z(layers: &[crate::Layer]) -> i32 {
    layers
        .iter()
        .map(crate::Layer::print_z)
        .fold(0.0_f64, f64::max)
        .ceil() as i32
}
