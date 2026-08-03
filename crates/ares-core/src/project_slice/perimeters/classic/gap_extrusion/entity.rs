use super::super::materialize::ExtrusionPath;

#[derive(Debug, PartialEq)]
pub(in crate::project_slice) enum GapFillEntity {
    Path(ExtrusionPath),
    Loop(Vec<ExtrusionPath>),
}

#[derive(Debug, Default, PartialEq)]
pub(in crate::project_slice) struct GapFillCollection {
    pub(in crate::project_slice) entities: Vec<GapFillEntity>,
}
