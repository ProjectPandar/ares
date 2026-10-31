//! Preset and archive construction helpers shared by runner parity cases.

use super::{FailureKind, Map, Path, PathBuf, Stage, StageError, Value};

#[path = "preset_helpers.rs"]
pub(crate) mod preset_helpers;
#[path = "volume_fixture.rs"]
pub(crate) mod volume_fixture;
#[path = "wrapping_input.rs"]
pub(crate) mod wrapping_input;
