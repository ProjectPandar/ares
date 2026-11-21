use crate::options::{GCodeFlavor, power_loss_recovery::PowerLossRecoveryMode};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PowerLossRecoveryState {
    emitted_second_layer_enable: bool,
}

/// `GCode.cpp:4777-4779`: the first non-first layer emits the configured
/// mode's command through the `GCodeWriter.cpp:505-522` dispatch.
pub(crate) fn second_layer_command(
    is_bbl: bool,
    flavor: GCodeFlavor,
    mode: PowerLossRecoveryMode,
    comments: bool,
    state: &mut PowerLossRecoveryState,
) -> String {
    let command = command(is_bbl, flavor, mode, comments);
    if !command.is_empty() && mode == PowerLossRecoveryMode::Enable {
        state.emitted_second_layer_enable = true;
    }
    command
}

/// `GCode.cpp:3440-3441` (ByLayer layer-loop end) and the per-object
/// completion `:3359-3361`: after the second-layer block ran with
/// `Enable`, the print end emits the disable command.
pub(crate) fn finish_command(
    is_bbl: bool,
    flavor: GCodeFlavor,
    comments: bool,
    state: PowerLossRecoveryState,
) -> String {
    if state.emitted_second_layer_enable {
        command(is_bbl, flavor, PowerLossRecoveryMode::Disable, comments)
    } else {
        String::new()
    }
}

/// `GCodeWriter.cpp:505-522`: `PrinterConfiguration` emits nothing;
/// BBL machines spell the command `M1003 S1`/`M1003 S0`, Marlin 2
/// firmware `M413 S1`/`M413 S0`, every other flavor nothing, and
/// `GCodeWriter::full_gcode_comment` appends the verbatim
/// ` ; set Power-loss Recovery` suffix.
fn command(
    is_bbl: bool,
    flavor: GCodeFlavor,
    mode: PowerLossRecoveryMode,
    comments: bool,
) -> String {
    if mode == PowerLossRecoveryMode::PrinterConfiguration
        || (!is_bbl && flavor != GCodeFlavor::MarlinFirmware)
    {
        return String::new();
    }
    let enabled = if mode == PowerLossRecoveryMode::Enable {
        "1"
    } else {
        "0"
    };
    let code = if is_bbl { "M1003" } else { "M413" };
    let comment = if comments {
        " ; set Power-loss Recovery"
    } else {
        ""
    };
    format!("{code} S{enabled}{comment}\n")
}
