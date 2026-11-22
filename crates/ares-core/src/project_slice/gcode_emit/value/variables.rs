use std::collections::HashMap;

use super::Value;

/// The dictionary selected by a script declaration (`PlaceholderParser.cpp:1529-1564`).
#[derive(Clone, Copy, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum Scope {
    Local,
    Global,
}

/// The type of a vector variable: the four a script creates, and those it only reads.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::project_slice::gcode_emit) enum VectorKind {
    Floats,
    Ints,
    Strings,
    Bools,
    Percents,
    /// `coFloatsOrPercents`, `coEnums`, `coPoints` and `coPointsGroups`.
    Other,
}

#[derive(Clone, Debug)]
pub(in crate::project_slice::gcode_emit) struct Variable {
    pub(super) value: Value,
    pub(super) kind: Option<VectorKind>,
}

pub(in crate::project_slice::gcode_emit) type Variables = HashMap<String, Variable>;
