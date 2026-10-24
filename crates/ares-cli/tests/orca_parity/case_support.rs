//! Shared construction, outcome, inventory and smoke-option helpers for the parity suite.

use crate::{BTreeMap, Runner, Value};
use case_helpers::ParityOutcome;

#[path = "case_helpers.rs"]
pub(crate) mod case_helpers;
#[path = "outcomes.rs"]
pub(crate) mod outcomes;
#[path = "smoke_overrides.rs"]
pub(crate) mod smoke_overrides;
#[path = "vendor_inventory.rs"]
pub(crate) mod vendor_inventory;

#[path = "startup_fan_initial_tool.rs"]
mod startup_fan_initial_tool;

#[path = "absolute_e_reset_comment.rs"]
mod absolute_e_reset_comment;
