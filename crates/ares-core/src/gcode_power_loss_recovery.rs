use crate::options::{GCodeFlavor, power_loss_recovery::PowerLossRecoveryMode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PowerLossRecoveryState {
    emitted_second_layer_enable: bool,
}

impl PowerLossRecoveryState {
    pub(crate) const fn new() -> Self {
        Self {
            emitted_second_layer_enable: false,
        }
    }
}

pub(crate) fn layer_command(
    flavor: GCodeFlavor,
    mode: PowerLossRecoveryMode,
    layer_index: usize,
    comments: bool,
    state: &mut PowerLossRecoveryState,
) -> String {
    if layer_index != 1 {
        return String::new();
    }
    let command = command(flavor, mode, comments);
    if !command.is_empty() && mode == PowerLossRecoveryMode::Enable {
        state.emitted_second_layer_enable = true;
    }
    command
}

pub(crate) fn finish_command(
    flavor: GCodeFlavor,
    comments: bool,
    state: PowerLossRecoveryState,
) -> String {
    if state.emitted_second_layer_enable {
        command(flavor, PowerLossRecoveryMode::Disable, comments)
    } else {
        String::new()
    }
}

fn command(flavor: GCodeFlavor, mode: PowerLossRecoveryMode, comments: bool) -> String {
    if mode == PowerLossRecoveryMode::PrinterConfiguration || flavor != GCodeFlavor::MarlinFirmware
    {
        return String::new();
    }
    let enabled = if mode == PowerLossRecoveryMode::Enable {
        "1"
    } else {
        "0"
    };
    let comment = if comments {
        " ; set Power-loss Recovery"
    } else {
        ""
    };
    format!("M413 S{enabled}{comment}\n")
}
