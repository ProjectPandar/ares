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

    /// `record_exit_reson` (`OrcaSlicer.cpp:535-586`): Linux-only, sorted
    /// keys, tab-indented `dump(1, '\t')` with a trailing newline; failures
    /// to write are swallowed like the upstream `catch (...)`. `warnings` and
    /// `strict_mode` are always written (`:573-574`); `--strict` is not
    /// ported, so `strict_mode` is false.
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
        let warnings = self.warnings_array()?;
        let body = format!(
            "{{\n\t\"error_string\": {json_error_string},\n\t\"export_time\": 0,\n\t\
             \"layer_height\": {:?},\n\t\"plate_index\": {},\n\t\"prepare_time\": 0,\n\t\
             \"return_code\": {},\n\t\"sparse_infill_density\": {:?},\n\t\"strict_mode\": false,\n\
             {upward}\t\"wall_loops\": {},\n\t\"warnings\": {warnings}\n}}\n",
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

    /// `j["warnings"]` as `dump(1, '\t')` writes it: `[]`, or one sorted-key object per entry.
    #[cfg(target_os = "linux")]
    fn warnings_array(&self) -> Result<String, serde_json::Error> {
        if self.warnings.is_empty() {
            return Ok("[]".to_owned());
        }
        let entries = self
            .warnings
            .iter()
            .map(|(file, substitution)| {
                Ok(format!(
                    "\t\t{{\n\t\t\t\"class\": \"config_value_substituted\",\n\t\t\t\"file\": {},\n\t\t\t\
                     \"key\": {},\n\t\t\t\"new_value\": {},\n\t\t\t\"old_value\": {}\n\t\t}}",
                    serde_json::to_string(file)?,
                    serde_json::to_string(&substitution.key)?,
                    serde_json::to_string(&substitution.new_value)?,
                    serde_json::to_string(&substitution.old_value)?,
                ))
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;
        Ok(format!("[\n{}\n\t]", entries.join(",\n")))
    }
}
