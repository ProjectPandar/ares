pub(crate) fn render(template: String) -> String {
    template
        .replace(
            "[print_time_sec]",
            crate::gcode_reserved_tags::PRINT_TIME_SEC,
        )
        .replace(
            "[used_filament_length]",
            crate::gcode_reserved_tags::USED_FILAMENT_LENGTH,
        )
}
