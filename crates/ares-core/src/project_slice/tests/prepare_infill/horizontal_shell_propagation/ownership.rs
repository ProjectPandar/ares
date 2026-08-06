mod snapshots;

use crate::project_slice::{
    prepare_infill::horizontal_shell_propagation::{self, PropagationEvent},
    tests::support::KsrArchive,
};

#[test]
fn task22o26_active_success_moves_sidecars_and_replaces_only_dirty_fill_graphs() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"ensure_moderate\"",
    );
    let input = super::fixture::prepare_o25(archive.bytes());
    let predecessor = std::ptr::from_ref(input.predecessor.as_ref());
    let outer = [
        input.objects.as_ptr() as usize,
        input.caches.as_ptr() as usize,
        input.projections.as_ptr() as usize,
        input.trims.as_ptr() as usize,
        input.regularizations.as_ptr() as usize,
        input.filters.as_ptr() as usize,
    ];
    let before_records = snapshots::records(&input.objects);
    let before_sidecars = snapshots::sidecars(
        &input.caches,
        &input.projections,
        &input.trims,
        &input.regularizations,
        &input.filters,
    );

    horizontal_shell_propagation::reset_hooks();
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let mut dirty = vec![false; before_records.len()];
    for event in horizontal_shell_propagation::events() {
        if let PropagationEvent::DirtyCommit { object, layer } = event {
            assert_eq!(object, 0);
            dirty[layer] = true;
        }
    }
    assert!(dirty.iter().any(|dirty| *dirty));

    assert_eq!(std::ptr::from_ref(output.predecessor.as_ref()), predecessor);
    assert_eq!(
        [
            output.objects.as_ptr() as usize,
            output.caches.as_ptr() as usize,
            output.projections.as_ptr() as usize,
            output.trims.as_ptr() as usize,
            output.regularizations.as_ptr() as usize,
            output.filters.as_ptr() as usize,
        ],
        outer
    );
    assert_eq!(
        snapshots::sidecars(
            &output.caches,
            &output.projections,
            &output.trims,
            &output.regularizations,
            &output.filters,
        ),
        before_sidecars
    );

    let after_records = snapshots::records(&output.objects);
    for (index, (before, after)) in before_records.iter().zip(&after_records).enumerate() {
        let (Some(before), Some(after)) = (before, after) else {
            assert_eq!(before.is_none(), after.is_none());
            continue;
        };
        for field in [0, 1, 2, 4, 5] {
            assert_eq!(after.fields[field], before.fields[field]);
        }
        if dirty[index] {
            assert_ne!(after.fields[3].0, before.fields[3].0);
            let before_points = before
                .fill_points
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>();
            assert!(
                after
                    .fill_points
                    .iter()
                    .flatten()
                    .all(|pointer| !before_points.contains(pointer))
            );
        } else {
            assert_eq!(after.fields[3], before.fields[3]);
            assert_eq!(after.fill_points, before.fill_points);
            assert_eq!(after.fill_content, before.fill_content);
        }
    }
    horizontal_shell_propagation::dispose(output);
}
