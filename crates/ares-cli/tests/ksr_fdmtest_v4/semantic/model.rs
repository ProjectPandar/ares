#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Position {
    pub(super) x: String,
    pub(super) y: String,
    pub(super) z: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct MotionRecord {
    pub(super) command: String,
    pub(super) start: Position,
    pub(super) end: Position,
    pub(super) arc_center: [Option<String>; 2],
    pub(super) turns: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum LifecycleEvent {
    Extruder {
        extrusion: String,
        feed: String,
    },
    WipeStart,
    Wipe {
        motion: Box<MotionRecord>,
        extrusion: String,
        feed: String,
    },
    WipeEnd,
}
