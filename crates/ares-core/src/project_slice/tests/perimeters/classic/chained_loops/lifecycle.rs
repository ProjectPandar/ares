use crate::project_slice::perimeters::prepare_post_classic_chained_loops;

use super::super::super::super::support::ksr_project;

#[test]
fn task22o8_code_level_preparation_constructs_loops_before_terminal_boundary() {
    let prepared = prepare_post_classic_chained_loops(ksr_project()).unwrap();
    assert!(prepared.objects.iter().any(|object| {
        object.records.iter().flatten().any(|record| {
            record.surfaces.iter().any(|surface| {
                surface
                    .roots
                    .iter()
                    .any(|root| root.extrusion_loop.is_some())
            })
        })
    }));
}
