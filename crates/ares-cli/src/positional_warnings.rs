use std::path::{Component, Path, PathBuf};

use ares_core::ConfigSubstitution;

#[derive(Default)]
pub(super) struct PositionalWarnings {
    #[cfg(target_os = "linux")]
    report: Option<(PathBuf, Vec<crate::result_json::CliWarning>)>,
}

impl PositionalWarnings {
    /// OrcaSlicer.cpp:1992: report the admitted positional project's load substitutions
    /// before any action runs. Linux keeps the same ordered records for the exit report.
    pub(super) fn record(
        &mut self,
        input: &Path,
        _output: &Path,
        substitutions: &[ConfigSubstitution],
    ) {
        if substitutions.is_empty() {
            return;
        }
        let path = resolve_input(input);
        let file = path.to_string_lossy();
        eprint!(
            "{}",
            crate::failure::substitution_warnings(&file, substitutions)
        );
        #[cfg(target_os = "linux")]
        {
            self.report = Some((
                _output.to_owned(),
                crate::result_json::substitution_records(&file, substitutions.iter().cloned())
                    .collect(),
            ));
        }
    }

    #[cfg(target_os = "linux")]
    pub(super) fn take(&mut self) -> Vec<crate::result_json::CliWarning> {
        self.report
            .take()
            .map_or_else(Vec::new, |(_, records)| records)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn attach(self, failure: crate::CliFailure, strict_mode: bool) -> crate::CliFailure {
        match self.report {
            Some((output, warnings)) => failure.with_load_warnings(output, warnings, strict_mode),
            None => failure,
        }
    }
}

// resolve_cli_input_path (libslic3r/utils.cpp:1577–1588): never canonicalize an
// absolute argument or consult the filesystem when collapsing relative components.
fn resolve_input(input: &Path) -> PathBuf {
    if input.is_absolute() {
        return input.to_owned();
    }
    let Ok(directory) = std::env::current_dir() else {
        return input.to_owned();
    };
    let mut result = PathBuf::new();
    for component in directory.join(input).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component),
        }
    }
    result
}
