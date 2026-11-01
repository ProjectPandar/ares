//! Processor time-machine parity group: M73 progress placement
//! (`GCodeProcessor.cpp:748-754` cache, `:1241-1252` export probe) and the
//! estimated-printing-time placeholder tag suite. Registration split from
//! `runner.rs` at the 400-LOC module cap; test files are unchanged.

#[path = "../m73_progress_placement.rs"]
mod m73_progress_placement;

#[path = "placeholder.rs"]
mod placeholder;
