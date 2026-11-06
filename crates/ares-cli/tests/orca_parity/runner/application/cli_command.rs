use std::{
    ffi::OsStr,
    io,
    process::{Command, Output},
};

pub(crate) struct CliCommand(pub(super) Command);

pub(crate) fn cli_command() -> CliCommand {
    CliCommand::new(env!("CARGO_BIN_EXE_ares"))
}

impl CliCommand {
    pub(crate) fn new(binary: impl AsRef<OsStr>) -> Self {
        Self(Command::new(binary))
    }

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

    pub(crate) fn output(&mut self) -> io::Result<Output> {
        let explicit = self.0.get_args().any(|arg| {
            arg.to_str().is_some_and(|arg| {
                matches!(
                    arg.split_once('=').map_or(arg, |(flag, _)| flag),
                    "--project-preset-catalog"
                        | "--load-settings"
                        | "--load-filaments"
                        | "--resources-dir"
                        | "--uptodate"
                )
            })
        });
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
