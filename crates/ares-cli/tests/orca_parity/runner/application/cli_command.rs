use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Command, Output},
};

pub(crate) struct CliCommand(Command);

pub(crate) fn cli_command() -> CliCommand {
    CliCommand(Command::new(env!("CARGO_BIN_EXE_ares")))
}

impl CliCommand {
    pub(crate) fn arg(&mut self, arg: impl AsRef<OsStr>) -> &mut Self {
        self.0.arg(arg);
        self
    }

    pub(crate) fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.0.args(args);
        self
    }

    pub(crate) fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.0.current_dir(path);
        self
    }

    pub(crate) fn output(&mut self) -> io::Result<Output> {
        // An explicitly supplied catalog is already a project-derived request.
        let explicit = self
            .0
            .get_args()
            .any(|arg| arg == "--project-preset-catalog");
        if !explicit
            && let Some(supply) = super::project_presets::project_supply()
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))?
        {
            self.0
                .arg("--project-preset-catalog")
                .arg(&supply.catalog_path);
        }
        self.0.output()
    }
}
