use super::super::estimate::options_z::MoveIds;
use super::super::motion::Trapezoid;
use super::MoveRecorder;

impl MoveRecorder<'_> {
    pub(in crate::project_slice::gcode_emit::processor) fn next_vertex(&self) -> MoveIds {
        self.options_z.next_vertex(self.moves.len())
    }

    pub(in crate::project_slice::gcode_emit::processor) fn queue_block(&mut self, ids: MoveIds) {
        self.options_z.block(ids);
    }

    pub(in crate::project_slice::gcode_emit::processor) fn queue_virtual(&mut self) {
        self.options_z.virtual_block();
    }

    pub(in crate::project_slice::gcode_emit::processor) fn apply_batch(
        &mut self,
        batch: &[(f64, Trapezoid)],
    ) -> Vec<Option<bool>> {
        self.options_z.flush(&mut self.moves, batch)
    }

    /// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`).
    pub(super) fn correct_options_z(&mut self) {
        if let Some(item_id) = self.pending_option.take() {
            let position = self.moves[self.moves.len() - 1].position;
            self.options_z.update(&mut self.moves, self.height);
            self.custom_gcode_per_print_z[item_id].print_z = f64::from(position[2]);
        }
    }

    pub(super) fn set_option(&mut self) {
        self.pending_option = Some(self.custom_gcode_per_print_z.len() - 1);
        self.options_z.set(self.moves.len() - 1);
    }
}
