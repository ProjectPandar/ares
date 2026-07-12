use crate::{SliceError, SliceOptions};

pub(crate) fn layer_command(
    options: &SliceOptions,
    layer_index: usize,
) -> Result<&'static str, SliceError> {
    let scan_first_layer = options.bool_option("scan_first_layer", false)?;
    if layer_index != 1 || !scan_first_layer {
        return Ok("");
    }
    if !is_bambu_lab_printer(options) {
        return Ok("");
    }

    Ok("M976 S1 P1 ; scan model before printing 2nd layer\nM400 P100\n")
}

fn is_bambu_lab_printer(options: &SliceOptions) -> bool {
    options
        .values()
        .get("printer_model")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|printer_model| printer_model.starts_with("Bambu Lab"))
}
