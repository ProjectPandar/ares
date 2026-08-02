use crate::{
    SliceError, project_slice::perimeters::classic::top_split::TopSplitOutcome,
    project_slice::perimeters::prepare_post_classic_top_split, slice_project,
};

use super::support::{archive, geometry_summary, metadata, outcomes, project};

const NORMAL_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"256x0\",\r\n",
    "\t\t\"256x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);
const LARGE_PRINTABLE_AREA: &str = concat!(
    "\t\"printable_area\": [\r\n",
    "\t\t\"0x0\",\r\n",
    "\t\t\"2148x0\",\r\n",
    "\t\t\"2148x256\",\r\n",
    "\t\t\"0x256\"\r\n",
    "\t]",
);

#[test]
fn task22o2_ksr_has_applied_typed_top_split_outputs() {
    let outcomes = outcomes(project());
    assert!(!outcomes.is_empty());
    assert!(outcomes.contains(&TopSplitOutcome::Applied));
    assert!(
        geometry_summary(project())
            .iter()
            .any(|value| value.2 > 0 && value.3 > 0)
    );
}

#[test]
fn task22o2_ksr_executes_at_both_supported_coordinate_scales() {
    let normal = prepare_post_classic_top_split(project()).unwrap();
    let mut large = archive();
    large.replace_unique(
        "Metadata/project_settings.config",
        NORMAL_PRINTABLE_AREA,
        LARGE_PRINTABLE_AREA,
    );
    let large = prepare_post_classic_top_split(large.bytes()).unwrap();
    assert_ne!(normal.scale, large.scale);
    assert_eq!(normal.objects.len(), large.objects.len());
    assert!(
        large
            .objects
            .into_iter()
            .flat_map(|object| object.into_parts().1)
            .flatten()
            .flat_map(|record| record.surfaces)
            .any(|surface| surface.outcome == TopSplitOutcome::Applied)
    );
}

#[tokio::test]
async fn task22o2_ksr_public_lifecycle_executes_stage_then_stays_incomplete() {
    assert_eq!(
        slice_project(project(), metadata()).await.unwrap_err(),
        SliceError::ProjectSlicingIncomplete
    );
}
