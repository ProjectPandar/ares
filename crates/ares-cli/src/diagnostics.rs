use std::{ffi::OsString, fmt, fs, io::Write, path::Path};

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
    #[arg(skip)]
    file: Option<fs::File>,
}

impl DiagnosticLog {
    pub(crate) fn startup_version(&mut self) {
        // OrcaSlicer.cpp:1524–1538: a warning after CLI log setup, before input loading.
        self.warning(format_args!(
            "cli mode, Current OrcaSlicer Version {}",
            ares_core::ORCA_SLICER_COMPATIBILITY_VERSION
        ));
    }

    pub(crate) fn warning(&mut self, message: fmt::Arguments<'_>) {
        if self.debug >= 2 {
            self.record(message);
        }
    }

    pub(crate) fn error(&mut self, message: fmt::Arguments<'_>) {
        // utils.cpp:160–200: level zero accepts fatal messages, but not errors.
        if self.debug != 0 {
            self.record(message);
        }
    }

    fn record(&mut self, message: fmt::Arguments<'_>) {
        if self.file.is_none() {
            self.file = self.open_file();
        }
        if let Some(file) = &mut self.file {
            let _ = writeln!(file, "{message}");
        }
    }

    fn open_file(&self) -> Option<fs::File> {
        let path = self
            .logfile
            .as_deref()
            .filter(|path| !path.is_empty())
            .map(Path::new)
            .unwrap_or_else(|| Path::new("00000.log"));
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).ok()?;
        }
        fs::File::create(path).ok()
    }
}
