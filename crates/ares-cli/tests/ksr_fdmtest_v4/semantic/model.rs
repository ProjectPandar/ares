#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Position {
    pub(crate) x: String,
    pub(crate) y: String,
    pub(crate) z: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MotionRecord {
    pub(crate) command: String,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) arc_center: [Option<String>; 2],
    pub(crate) turns: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum LifecycleEvent {
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
