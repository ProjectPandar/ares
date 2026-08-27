use crate::project_slice::{
    perimeters,
    prepare_infill::{fill_surfaces, surface_type_detection},
    region_slices::{RegionSurface, RegionSurfaceKind},
    tests::support::KsrArchive,
};

use super::{fixture, ksr, ownership};

type Transitions = [[usize; 5]; 5];

fn kind_index(kind: RegionSurfaceKind) -> usize {
    match kind {
        RegionSurfaceKind::Top => 0,
        RegionSurfaceKind::Bottom => 1,
        RegionSurfaceKind::BottomBridge => 2,
        RegionSurfaceKind::Internal => 3,
        RegionSurfaceKind::InternalSolid => 4,
        RegionSurfaceKind::InternalBridge => panic!("O18 has no internal-bridge producer"),
        RegionSurfaceKind::InternalVoid => panic!("O18 has no internal-void producer"),
    }
}

fn transition_counts(before: &[RegionSurfaceKind], after: &[RegionSurface]) -> Transitions {
    assert_eq!(before.len(), after.len());
    let mut counts = [[0; 5]; 5];
    for (before, after) in before.iter().zip(after) {
        counts[kind_index(*before)][kind_index(after.as_parts().0)] += 1;
    }
    counts
}

fn add_transitions(output: &mut Transitions, counts: Transitions) {
    for (output, counts) in output.iter_mut().zip(counts) {
        for (output, count) in output.iter_mut().zip(counts) {
            *output += count;
        }
    }
}

fn archive_with_global(key: &str, from: &str, to: &str) -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        &format!("\"{key}\": \"{from}\""),
        &format!("\"{key}\": \"{to}\""),
    );
    archive
}

fn detected(archive: &KsrArchive) -> surface_type_detection::PreparedPostSurfaceTypeDetection {
    surface_type_detection::prepare(
        perimeters::prepare_post_layer_region_perimeters(&archive.clone().bytes()).unwrap(),
    )
    .unwrap()
}

fn all_fill_transitions(archive: &KsrArchive) -> Transitions {
    let before = detected(archive);
    let predecessor = std::ptr::from_ref(before.predecessor.as_ref());
    let allocations = ownership::allocation_snapshot(&before.objects);
    let unrelated = ksr::unrelated_checksum(&before.predecessor, &before.objects);
    let kinds = before
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .map(|record| {
                    record.as_ref().map(|record| {
                        record
                            .fill_surfaces
                            .iter()
                            .map(|surface| surface.as_parts().0)
                            .collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let after = fill_surfaces::prepare(before);
    assert_eq!(std::ptr::from_ref(after.predecessor.as_ref()), predecessor);
    assert_eq!(ownership::allocation_snapshot(&after.objects), allocations);
    assert_eq!(
        ksr::unrelated_checksum(&after.predecessor, &after.objects),
        unrelated
    );

    let mut output = [[0; 5]; 5];
    for (before, after) in kinds.iter().zip(&after.objects) {
        for (before, after) in before.iter().zip(&after.records) {
            match (before, after) {
                (Some(before), Some(after)) => {
                    add_transitions(&mut output, transition_counts(before, &after.fill_surfaces))
                }
                (None, None) => {}
                _ => panic!("O18 option records remain aligned"),
            }
        }
    }
    output
}

#[test]
fn task22o18_real_3mf_options_drive_nonzero_literal_transitions() {
    let top = all_fill_transitions(&archive_with_global("top_shell_layers", "5", "0"));
    let bottom = all_fill_transitions(&archive_with_global("bottom_shell_layers", "3", "0"));
    let density =
        all_fill_transitions(&archive_with_global("sparse_infill_density", "15%", "100%"));
    assert_eq!(
        top,
        [
            [0, 0, 0, 113, 0],
            [0, 6, 0, 0, 0],
            [0, 0, 48, 0, 0],
            [0, 0, 0, 1_127, 0],
            [0; 5],
        ]
    );
    assert_eq!(
        bottom,
        [
            [113, 0, 0, 0, 0],
            [0, 0, 0, 6, 0],
            [0, 0, 0, 48, 0],
            [0, 0, 0, 1_127, 0],
            [0; 5],
        ]
    );
    assert_eq!(
        density,
        [
            [113, 0, 0, 0, 0],
            [0, 6, 0, 0, 0],
            [0, 0, 48, 0, 0],
            [0, 0, 0, 0, 1_127],
            [0; 5],
        ]
    );

    let mut combined = KsrArchive::new();
    for (key, from, to) in [
        ("top_shell_layers", "5", "0"),
        ("bottom_shell_layers", "3", "0"),
        ("sparse_infill_density", "15%", "100%"),
    ] {
        combined.replace_unique(
            "Metadata/project_settings.config",
            &format!("\"{key}\": \"{from}\""),
            &format!("\"{key}\": \"{to}\""),
        );
    }
    assert_eq!(
        all_fill_transitions(&combined),
        [
            [0, 0, 0, 0, 113],
            [0, 0, 0, 0, 6],
            [0, 0, 0, 0, 48],
            [0, 0, 0, 0, 1_127],
            [0; 5],
        ]
    );
}

#[test]
fn task22o18_real_3mf_density_obeys_strict_source_epsilon() {
    let inside = all_fill_transitions(&archive_with_global(
        "sparse_infill_density",
        "15%",
        "99.99995%",
    ));
    let outside = all_fill_transitions(&archive_with_global(
        "sparse_infill_density",
        "15%",
        "99.9998%",
    ));
    assert_eq!(inside[3][4], 1_127);
    assert_eq!(outside[3][4], 0);
}

#[test]
fn task22o18_region_override_and_aligned_objects_do_not_use_global_shortcuts() {
    let mut override_archive = KsrArchive::new();
    override_archive.replace(
        "Metadata/model_settings.config",
        r#"<part id="1" subtype="normal_part">"#,
        r#"<part id="1" subtype="normal_part"><metadata key="top_shell_layers" value="0"/>"#,
    );
    let override_output = fixture::prepare(override_archive.clone().bytes());
    assert_eq!(
        override_output
            .predecessor
            .resolved
            .views
            .full
            .process
            .region
            .top_shell_layers
            .0,
        5
    );

    let mut first = detected(&KsrArchive::new());
    let mut second = detected(&override_archive);
    first.objects.push(second.objects.pop().unwrap());
    first
        .predecessor
        .objects
        .push(second.predecessor.objects.pop().unwrap());
    let before = first
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .flatten()
                .flat_map(|record| &record.fill_surfaces)
                .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
                .count()
        })
        .collect::<Vec<_>>();
    let output = fill_surfaces::prepare(first);
    let after = output
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .flatten()
                .flat_map(|record| &record.fill_surfaces)
                .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
                .count()
        })
        .collect::<Vec<_>>();
    assert_eq!(before, [113, 113]);
    assert_eq!(after, [113, 0]);
}
