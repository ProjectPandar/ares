use std::{fs, path::Path};

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{compare_sliced_archives, invoke, write_saved_input};
use crate::OrcaRunner;

#[test]
fn sliced_3mf_uses_last_assemble_item_for_an_instance() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(source.join("out")).unwrap();
    write_saved_input(&runner, &source);
    let project = rewrite_project(&fs::read(source.join("saved.3mf")).unwrap(), |name, xml| {
        if name != "Metadata/model_settings.config" {
            return xml.to_owned();
        }
        assert!(xml.contains("  </assemble>"));
        xml.replace(
            "  </assemble>",
            concat!(
                "   <assemble_item object_id=\"2\" instance_id=\"0\" transform=\"1 0 0 0 1 0 0 0 1 0 0 60\" offset=\"9 8 7\" />\n",
                "   <assemble_item object_id=\"2\" volume_id=\"19\" transform=\"1 0 0 0 1 0 0 0 1 -0.2 0.0000125 -7.2\" />\n",
                "   <assemble_item object_id=\"2\" instance_id=\"1\" transform=\"1 0 0 0 1 0 0 0 1 0 0 30\" offset=\"6 5 4\" />\n",
                "  </assemble>",
            ),
        )
    });
    let native = root.join("native/cwd");
    let ares = root.join("ares/cwd");
    let argv = ["--slice", "1", "--export-3mf", "export.3mf", "saved.3mf"];
    let variants: [(&Path, &Path); 2] = [
        (runner.bin(), &native),
        (Path::new(env!("CARGO_BIN_EXE_ares")), &ares),
    ];
    for (bin, cwd) in variants {
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join("saved.3mf"), &project).unwrap();
        assert_eq!(invoke(bin, cwd, &argv), Some(0), "{}", cwd.display());
    }
    compare_sliced_archives(&native.join("export.3mf"), &ares.join("export.3mf"));
}
