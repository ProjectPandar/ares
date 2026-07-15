use crate::{PrinterModel, ProjectSettings, options::config_export::is_bambu_project};

#[test]
fn config_export_bambu_model_prefix_is_exact_and_case_sensitive() {
    for model in ["Bambu Lab", "Bambu Lab X1 Carbon", "Bambu Laboratory"] {
        let mut settings = ProjectSettings::default();
        settings.printer.remaining.printer_model = PrinterModel(model.to_owned());
        assert!(is_bambu_project(&settings), "{model}");
    }

    for model in [
        "",
        "bambu Lab X1 Carbon",
        "Bambu lab X1 Carbon",
        " Bambu Lab X1 Carbon",
        "Other Bambu Lab X1 Carbon",
    ] {
        let mut settings = ProjectSettings::default();
        settings.printer.remaining.printer_model = PrinterModel(model.to_owned());
        assert!(!is_bambu_project(&settings), "{model}");
    }
}
