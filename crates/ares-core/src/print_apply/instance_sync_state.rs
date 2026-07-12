use serde_json::Value;

use super::{
    Point2, PrintableFilamentGeometryOps, ScaledPoint, SliceError,
    printable_filament_changed_staged,
};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct StagedInstanceApplyState {
    pub(super) convex_hull: Vec<Point2>,
    pub(super) transform: i32,
    pub(super) print_volume_state: i32,
    pub(super) printable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StagedPrintStep {
    WipeTower,
    GCodeExport,
}

pub(super) fn sync_changed_instance_printable_filament_staged<D, A, I>(
    old_instance: &mut StagedInstanceApplyState,
    new_instance: &StagedInstanceApplyState,
    new_full_config_values: &serde_json::Map<String, Value>,
    ops: PrintableFilamentGeometryOps<D, A, I>,
) -> Result<Vec<StagedPrintStep>, SliceError>
where
    D: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
    A: FnOnce(&[Vec<ScaledPoint>], &[Vec<ScaledPoint>]) -> Vec<Vec<ScaledPoint>>,
    I: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
{
    let changed = printable_filament_changed_staged(
        new_full_config_values,
        (&old_instance.convex_hull, &new_instance.convex_hull),
        ops.diff,
        ops.all_intersection,
        ops.intersection,
    )?;
    let steps = if changed {
        vec![StagedPrintStep::WipeTower, StagedPrintStep::GCodeExport]
    } else {
        Vec::new()
    };
    old_instance.transform = new_instance.transform;
    old_instance.print_volume_state = new_instance.print_volume_state;
    old_instance.printable = new_instance.printable;
    Ok(steps)
}
