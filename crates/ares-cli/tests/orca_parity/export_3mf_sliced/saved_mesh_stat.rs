use std::{fs, path::Path, str};

use regex::Regex;

use super::super::export_3mf_sliced_rejection::rewrite_project;
use super::{comparator, invoke, write_saved_input};
use crate::OrcaRunner;

const PROJECT: &str = "saved.3mf";
const EXPORT: &str = "export.3mf";
const MODEL: &str = "3D/3dmodel.model";
const SETTINGS: &str = "Metadata/model_settings.config";
const TRANSLATION: &str = "1 0 0 0 1 0 0 0 1 128 128 5";
const AFFINE: &str = "0.8 0.6 0 -0.6 0.8 0 0 0 1 128 128 5";
const ZERO: [u32; 5] = [0; 5];
const OWNER: [u32; 5] = [2, 3, 5, 7, 11];
const REUSE: [u32; 5] = [13, 17, 19, 23, 29];

#[test]
fn saved_affine_zero_mesh_stat_matches_native() {
    check(ZERO, AFFINE, false, false);
}

#[test]
fn saved_translation_nonzero_mesh_stat_matches_native() {
    check(OWNER, TRANSLATION, false, false);
}

#[test]
fn saved_affine_nonzero_mesh_stat_matches_native() {
    check(OWNER, AFFINE, false, false);
}

#[test]
fn saved_inward_mesh_changes_winding_but_not_mesh_stat() {
    check(OWNER, TRANSLATION, true, false);
}

#[test]
fn saved_shared_mesh_stat_comes_from_owner() {
    check(OWNER, TRANSLATION, false, true);
}

fn check(counts: [u32; 5], transform: &str, inward: bool, shared: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = runner.work_dir();
    let source = root.join("source/cwd");
    fs::create_dir_all(source.join("out")).unwrap();
    write_saved_input(&runner, &source);
    let saved = fs::read(source.join(PROJECT)).unwrap();
    let face = Regex::new(r#"<triangle v1="(\d+)" v2="(\d+)" v3="(\d+)"/>"#).unwrap();
    let outward = mesh_winding(&face, &comparator::read_members(&source.join(PROJECT)));
    assert!(!outward.is_empty());
    let project = rewrite_project(&saved, |name, xml| match name {
        MODEL => {
            assert_eq!(xml.matches(TRANSLATION).count(), 1);
            let xml = xml.replace(TRANSLATION, transform);
            if shared {
                duplicate_component(xml)
            } else {
                xml
            }
        }
        SETTINGS => patch_settings(xml, counts, shared),
        _ if inward && name.starts_with("3D/Objects/") && name.ends_with(".model") => face
            .replace_all(xml, r#"<triangle v1="$1" v2="$3" v3="$2"/>"#)
            .into_owned(),
        _ => xml.to_owned(),
    });
    let export = |bin: &Path, name: &str| {
        let cwd = root.join(name).join("cwd");
        fs::create_dir_all(cwd.join("out")).unwrap();
        fs::write(cwd.join(PROJECT), &project).unwrap();
        let argv = ["--slice", "1", "--export-3mf", EXPORT, PROJECT];
        assert_eq!(invoke(bin, &cwd, &argv), Some(0), "export in {cwd:?}");
        let outputs = fs::read_dir(&cwd)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .filter(|name| {
                name != PROJECT && matches!(name.to_str(), Some(name) if name.ends_with(".3mf") || name.ends_with(".gcode"))
            })
            .collect::<Vec<_>>();
        assert_eq!(outputs, [EXPORT]);
        assert_eq!(fs::read_dir(cwd.join("out")).unwrap().count(), 0);
        let members = comparator::read_members(&cwd.join(EXPORT));
        let settings = members.iter().find(|entry| entry.name == SETTINGS).unwrap();
        let stats = str::from_utf8(&settings.bytes)
            .unwrap()
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("<mesh_stat "))
            .collect::<Vec<_>>();
        assert_eq!(
            stats,
            vec![mesh_stat(counts); if shared { 2 } else { 1 }],
            "live mesh_stat in {cwd:?}"
        );
        if inward {
            assert_eq!(
                mesh_winding(&face, &members),
                outward,
                "outward winding in {cwd:?}"
            );
        }
        members
    };
    let native = export(runner.bin(), "native");
    let ares = export(Path::new(env!("CARGO_BIN_EXE_ares")), "ares");
    // The shared fixture exercises counter ownership only. The exporters
    // give the reused resource different part IDs.
    if !shared {
        comparator::compare_members(native, ares);
    }
}

fn mesh_winding(face: &Regex, members: &[comparator::Member]) -> Vec<[u32; 3]> {
    members
        .iter()
        .filter(|entry| entry.name.starts_with("3D/Objects/") && entry.name.ends_with(".model"))
        .flat_map(|entry| face.captures_iter(str::from_utf8(&entry.bytes).unwrap()))
        .map(|capture| [1, 2, 3].map(|index| capture[index].parse().unwrap()))
        .collect()
}

fn mesh_stat([edges, degenerate, removed, reversed, backwards]: [u32; 5]) -> String {
    format!(
        r#"<mesh_stat edges_fixed="{edges}" degenerate_facets="{degenerate}" facets_removed="{removed}" facets_reversed="{reversed}" backwards_edges="{backwards}"/>"#
    )
}

fn patch_settings(xml: &str, counts: [u32; 5], shared: bool) -> String {
    let zero = mesh_stat(ZERO);
    assert_eq!(xml.matches(&zero).count(), 1);
    let xml = xml.replace(&zero, &mesh_stat(counts));
    if !shared {
        return xml;
    }
    let start = xml.find("    <part ").unwrap();
    let end_tag = "    </part>\n";
    let end = xml.find(end_tag).unwrap() + end_tag.len();
    let later = xml[start..end].replace(&mesh_stat(counts), &mesh_stat(REUSE));
    format!("{}{}{}", &xml[..end], later, &xml[end..])
}

fn duplicate_component(xml: String) -> String {
    let component = xml
        .lines()
        .find(|line| line.contains("<component "))
        .unwrap();
    let identity = "1 0 0 0 1 0 0 0 1 0 0 0";
    assert_eq!(component.matches(identity).count(), 1);
    let later = component
        .replace(identity, "1 0 0 0 1 0 0 0 1 30 0 0")
        .replace("p:UUID=\"00010000-", "p:UUID=\"00010001-");
    xml.replace(component, &format!("{component}\n{later}"))
}
