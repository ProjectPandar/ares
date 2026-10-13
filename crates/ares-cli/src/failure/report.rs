use super::*;

impl FamilyFailure {
    pub(super) fn flush_and_exit(self) -> ! {
        #[cfg(target_os = "linux")]
        println!("run found error, return {}, exit...", self.return_code);
        #[cfg(not(target_os = "linux"))]
        println!("run found error, exit");
        eprint!("{}", self.stderr);
        #[cfg(target_os = "linux")]
        let _ = self.record_exit_reason();
        std::process::exit(self.return_code);
    }

    /// `record_exit_reson` (`OrcaSlicer.cpp:416-460`): Linux-only, sorted
    /// keys, tab-indented `dump(1, '\t')` with a trailing newline; failures
    /// to write are swallowed like the upstream `catch (...)`.
    #[cfg(target_os = "linux")]
    fn record_exit_reason(&self) -> Result<(), serde_json::Error> {
        let json_error_string = serde_json::to_string(self.error_string)?;
        let upward = if self.upward_machines.is_empty() {
            String::new()
        } else {
            use serde::Serialize;
            let mut bytes = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
            self.upward_machines
                .serialize(&mut serde_json::Serializer::with_formatter(
                    &mut bytes, formatter,
                ))?;
            let array = String::from_utf8(bytes).expect("JSON serializer emits UTF-8");
            format!(
                "\t\"upward_compatible_machine\": {},\n",
                array.replace('\n', "\n\t")
            )
        };
        let body = format!(
            "{{\n\t\"error_string\": {json_error_string},\n\t\"export_time\": 0,\n\t\
             \"layer_height\": {:?},\n\t\"plate_index\": {},\n\t\"prepare_time\": 0,\n\t\
             \"return_code\": {},\n\t\"sparse_infill_density\": {:?},\n{upward}\t\"wall_loops\": {}\n}}\n",
            self.layer_height,
            self.plate_index,
            self.return_code,
            self.sparse_infill_density,
            self.wall_loops,
        );
        let directory = self
            .output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let _ = fs::write(directory.join("result.json"), body);
        Ok(())
    }
}
