//! Brim.cpp:289-303: the geometric thermal cap is not the material reference.

use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use std::io::{Cursor, Read, Write};

const BASE: &[u8] =
    include_bytes!("../../../../tests/parity/per-object-skirt-brim-types/auto-multisland.3mf");

#[test]
fn auto_brim_small_diagonal() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    // 1×1×3 mm: 1.5*diagonal = 2.121... <5; the height>10 bump is inactive.
    // The speed term is 3.375, so using the material reference omits the brim.
    let project = small_pillars();
    let case = NativeCase::slice(&runner, &project, "auto-brim-small-diagonal-");
    assert!(case.native.contains(";TYPE:Brim\n"));
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
}

fn small_pillars() -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(BASE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mesh = regex::Regex::new("(?s)<mesh>.*?</mesh>").unwrap();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "3D/Objects/overhang.stl_1.model" {
            bytes = mesh
                .replace(&String::from_utf8(bytes).unwrap(), PILLAR)
                .into_owned()
                .into_bytes();
        } else if file.name() == "3D/3dmodel.model" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace("0.1 0 0 0 0.1 0 0 0 5", "1 0 0 0 1 0 0 0 1")
                .into_bytes();
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

const PILLAR: &str = r#"<mesh><vertices>
<vertex x="-0.5" y="-0.5" z="-7.5"/><vertex x="0.5" y="-0.5" z="-7.5"/>
<vertex x="0.5" y="0.5" z="-7.5"/><vertex x="-0.5" y="0.5" z="-7.5"/>
<vertex x="-0.5" y="-0.5" z="-4.5"/><vertex x="0.5" y="-0.5" z="-4.5"/>
<vertex x="0.5" y="0.5" z="-4.5"/><vertex x="-0.5" y="0.5" z="-4.5"/>
</vertices><triangles>
<triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/>
<triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/>
<triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/>
<triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/>
<triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/>
<triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/>
</triangles></mesh>"#;
