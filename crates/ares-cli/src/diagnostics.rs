use std::{ffi::OsString, fmt, fs, path::Path};

#[derive(clap::Args)]
pub(crate) struct DiagnosticLog {
    /// Diagnostic verbosity (0: fatal, 1: error, 2: warning, 3: info, 4: debug, 5: trace).
    #[arg(
        long,
        default_value_t = 1,
        allow_negative_numbers = true,
        value_name = "LEVEL"
    )]
    debug: i32,
    /// Write diagnostics to FILE instead of 00000.log in the working directory.
    #[arg(long, value_name = "FILE")]
    logfile: Option<OsString>,
}

impl DiagnosticLog {
    pub(crate) fn error(&self, message: fmt::Arguments<'_>) {
        // utils.cpp:160–200: level zero accepts fatal messages, but not errors.
        if self.debug == 0 {
            return;
        }
        let path = self
            .logfile
            .as_deref()
            .filter(|path| !path.is_empty())
            .map(Path::new)
            .unwrap_or_else(|| Path::new("00000.log"));
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            && fs::create_dir_all(parent).is_err()
        {
            return;
        }
        let _ = fs::write(path, format!("{message}\n"));
    }
}
