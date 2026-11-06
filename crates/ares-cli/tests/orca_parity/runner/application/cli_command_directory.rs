use std::path::Path;

impl super::cli_command::CliCommand {
    pub(crate) fn current_dir(&mut self, path: impl AsRef<Path>) -> &mut Self {
        self.0.current_dir(path);
        self
    }
}
