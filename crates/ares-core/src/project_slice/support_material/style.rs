//! Generator-local style resolution from `SupportParameters.hpp:190-204`.

use crate::{ProcessSupportStyle, ProcessSupportType};

pub(super) fn resolve(
    support_type: ProcessSupportType,
    style: ProcessSupportStyle,
) -> ProcessSupportStyle {
    use ProcessSupportStyle::{Default, Grid, Organic, Snug, TreeHybrid, TreeSlim, TreeStrong};
    let tree = match support_type {
        ProcessSupportType::NormalAuto | ProcessSupportType::NormalManual => false,
        ProcessSupportType::TreeAuto | ProcessSupportType::TreeManual => true,
    };
    match (tree, style) {
        (false, Grid | Snug) | (true, Organic | TreeHybrid | TreeSlim | TreeStrong) => style,
        (false, Default | Organic | TreeHybrid | TreeSlim | TreeStrong) => Grid,
        (true, Default | Grid | Snug) => Organic,
    }
}
