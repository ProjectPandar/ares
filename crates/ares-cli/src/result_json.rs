//! `record_exit_reson` (`OrcaSlicer.cpp:535-586` at v2.5.0-beta `345efec64`): the Linux-only
//! result.json of a failure or a success, with sorted keys and the tab-indented `dump(1, '\t')`
//! plus a trailing newline. Write failures are swallowed like the upstream `catch (...)`.

use std::{fs, path::Path};

use serde::Serialize;

/// One entry of `sliced_info.warnings` (`cli_record_warning`, `OrcaSlicer.cpp:505-518`).
pub(super) enum CliWarning {
    /// `cli_report_config_substitutions` (`:520-534`): a value the 3MF load replaced.
    ConfigValueSubstituted {
        file: String,
        substitution: ares_core::ConfigSubstitution,
    },
    /// `OrcaSlicer.cpp:7150-7160`: noncritical PrintObject step warnings.
    SlicingWarningNonCritical { plate_id: u32, text: String },
}

/// One `sliced_plates` entry (`sliced_plate_info_t`, `OrcaSlicer.cpp:245-251`). The slice times
/// are wall-clock seconds (`:7259-7260`), written as 0.
pub(super) struct SlicedPlateEntry {
    pub(super) id: u32,
    pub(super) triangle_count: usize,
    pub(super) warning_message: String,
}

pub(super) struct ResultJson<'a> {
    pub(super) return_code: i32,
    pub(super) error_string: &'a str,
    pub(super) plate_index: u32,
    pub(super) layer_height: f64,
    pub(super) wall_loops: i32,
    pub(super) sparse_infill_density: f64,
    pub(super) upward_machines: &'a [String],
    pub(super) downward_machines: &'a [String],
    pub(super) sliced_plates: &'a [SlicedPlateEntry],
    pub(super) warnings: &'a [CliWarning],
}

impl ResultJson<'_> {
    /// Writes result.json beside `output`. `warnings` and `strict_mode` are always written
    /// (`:573-574`); `--strict` is not ported, so `strict_mode` is false.
    pub(super) fn write(&self, output: &Path) -> Result<(), serde_json::Error> {
        let downward = if self.downward_machines.is_empty() {
            String::new()
        } else {
            format!(
                "\t\"downward_compatible_machine\": {},\n",
                pretty(&self.downward_machines)?.replace('\n', "\n\t")
            )
        };
        let upward = if self.upward_machines.is_empty() {
            String::new()
        } else {
            format!(
                "\t\"upward_compatible_machine\": {},\n",
                pretty(&self.upward_machines)?.replace('\n', "\n\t")
            )
        };
        let body = format!(
            "{{\n\t\"error_string\": {},\n\t\"export_time\": 0,\n\t\"layer_height\": {:?},\n\t\
             \"plate_index\": {},\n\t\"prepare_time\": 0,\n\t\"return_code\": {},\n{}\t\
             \"sparse_infill_density\": {:?},\n\t\"strict_mode\": false,\n{downward}{upward}\t\
             \"wall_loops\": {},\n\t\"warnings\": {}\n}}\n",
            serde_json::to_string(self.error_string)?,
            self.layer_height,
            self.plate_index,
            self.return_code,
            self.sliced_plates_entry()?,
            self.sparse_infill_density,
            self.wall_loops,
            self.warnings_array()?,
        );
        let directory = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let _ = fs::write(directory.join("result.json"), body);
        Ok(())
    }

    /// `j["sliced_plates"]`, only present once a plate is pushed (`:560-569`).
    fn sliced_plates_entry(&self) -> Result<String, serde_json::Error> {
        if self.sliced_plates.is_empty() {
            return Ok(String::new());
        }
        let entries = self
            .sliced_plates
            .iter()
            .map(|plate| {
                Ok(format!(
                    "\t\t{{\n\t\t\t\"id\": {},\n\t\t\t\"sliced_time\": 0,\n\t\t\t\
                     \"sliced_time_with_cache\": 0,\n\t\t\t\"triangle_count\": {},\n\t\t\t\
                     \"warning_message\": {}\n\t\t}}",
                    plate.id,
                    plate.triangle_count,
                    serde_json::to_string(&plate.warning_message)?
                ))
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;
        Ok(format!(
            "\t\"sliced_plates\": [\n{}\n\t],\n",
            entries.join(",\n")
        ))
    }

    /// `j["warnings"]`: `[]`, or one sorted-key object per entry.
    fn warnings_array(&self) -> Result<String, serde_json::Error> {
        if self.warnings.is_empty() {
            return Ok("[]".to_owned());
        }
        let entries = self
            .warnings
            .iter()
            .map(|warning| match warning {
                CliWarning::ConfigValueSubstituted { file, substitution } => Ok(format!(
                    "\t\t{{\n\t\t\t\"class\": \"config_value_substituted\",\n\t\t\t\"file\": {},\n\t\t\t\
                     \"key\": {},\n\t\t\t\"new_value\": {},\n\t\t\t\"old_value\": {}\n\t\t}}",
                    serde_json::to_string(file)?,
                    serde_json::to_string(&substitution.key)?,
                    serde_json::to_string(&substitution.new_value)?,
                    serde_json::to_string(&substitution.old_value)?,
                )),
                CliWarning::SlicingWarningNonCritical { plate_id, text } => Ok(format!(
                    "\t\t{{\n\t\t\t\"class\": \"slicing_warning_non_critical\",\n\t\t\t\
                     \"plate_id\": {plate_id},\n\t\t\t\"text\": {}\n\t\t}}",
                    serde_json::to_string(text)?,
                )),
            })
            .collect::<Result<Vec<_>, serde_json::Error>>()?;
        Ok(format!("[\n{}\n\t]", entries.join(",\n")))
    }
}

/// A successful 3MF slice of one plate: `record_exit_reson(outfile_dir, 0, plate_to_slice, …)`
/// at the end of the run (`OrcaSlicer.cpp:8126-8136`). `plate_to_slice` is the selected plate.
pub(super) struct Success<'a> {
    pub(super) output: &'a Path,
    pub(super) plate: u32,
    pub(super) plate_index: u32,
    pub(super) sliced_plate: ares_core::SlicedPlate,
    pub(super) upward_machines: &'a [String],
    pub(super) downward_machines: &'a [String],
}

impl Success<'_> {
    pub(super) fn write(self, file: &str, substitutions: Vec<ares_core::ConfigSubstitution>) {
        let warning_message = self
            .sliced_plate
            .warnings
            .last()
            .map(|warning| warning.message.clone())
            .unwrap_or_default();
        let warnings = substitutions
            .into_iter()
            .map(|substitution| CliWarning::ConfigValueSubstituted {
                file: file.to_owned(),
                substitution,
            })
            .chain(slicing_warnings(self.plate, self.sliced_plate.warnings))
            .collect::<Vec<_>>();
        let _ = ResultJson {
            return_code: 0,
            error_string: "Success.",
            plate_index: self.plate_index,
            layer_height: self.sliced_plate.layer_height,
            wall_loops: self.sliced_plate.wall_loops,
            sparse_infill_density: self.sliced_plate.sparse_infill_density,
            upward_machines: self.upward_machines,
            downward_machines: self.downward_machines,
            sliced_plates: &[SlicedPlateEntry {
                id: self.plate,
                triangle_count: self.sliced_plate.triangle_count,
                warning_message,
            }],
            warnings: &warnings,
        }
        .write(self.output);
    }
}

pub(super) fn slicing_warnings(
    plate_id: u32,
    warnings: Vec<ares_core::PrintWarning>,
) -> impl Iterator<Item = CliWarning> {
    warnings
        .into_iter()
        .filter_map(move |warning| match warning.level {
            ares_core::WarningLevel::NonCritical => Some(CliWarning::SlicingWarningNonCritical {
                plate_id,
                text: warning.message,
            }),
            ares_core::WarningLevel::Critical => None,
        })
}

/// `value` as `serde_json` pretty-prints it with tab indentation.
fn pretty(value: &impl Serialize) -> Result<String, serde_json::Error> {
    let mut bytes = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");
    value.serialize(&mut serde_json::Serializer::with_formatter(
        &mut bytes, formatter,
    ))?;
    Ok(String::from_utf8(bytes).expect("JSON serializer emits UTF-8"))
}
