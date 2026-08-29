use std::borrow::Cow;

pub(in crate::project_slice::gcode_emit) const PRINT_TIME_SEC_PLACEHOLDER: &str =
    "__ARES_PRINT_TIME_SEC__";
pub(in crate::project_slice::gcode_emit) const USED_FILAMENT_LENGTH_PLACEHOLDER: &str =
    "__ARES_USED_FILAMENT_LENGTH__";

pub(super) fn expand<'a>(
    line: &'a str,
    print_time_sec: &str,
    used_filament_length: &str,
) -> Cow<'a, str> {
    let mut expanded = Cow::Borrowed(line);
    if expanded.contains(PRINT_TIME_SEC_PLACEHOLDER) {
        expanded = Cow::Owned(expanded.replace(PRINT_TIME_SEC_PLACEHOLDER, print_time_sec));
    }
    if expanded.contains(USED_FILAMENT_LENGTH_PLACEHOLDER) {
        expanded =
            Cow::Owned(expanded.replace(USED_FILAMENT_LENGTH_PLACEHOLDER, used_filament_length));
    }
    expanded
}
