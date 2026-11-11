mod label_inventory;
pub(super) use label_inventory::{LabelInventory, LayerLabels};

use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) fn append_definitions(output: &mut Vec<u8>, traversal: &PreparedPostClassicTraversal) {
    let settings = &traversal.resolved.views.full;
    let flavor = settings.printer.gcode.gcode_flavor;
    // `set_object_info` gates on flavor + `exclude_object` and returns empty
    // for BBL printers (`GCode.cpp:8075-8077, 2697`).
    if !super::footprint::EXCLUDE_FLAVORS.contains(&flavor)
        || !settings.process.print.exclude_object.0
        || super::tags::Tags::of(traversal).is_bbl()
    {
        return;
    }
    let klipper = flavor == crate::GCodeFlavor::Klipper;
    let rrf = flavor == crate::GCodeFlavor::RepRapFirmware;
    for definition in &super::footprint::definitions(traversal) {
        definition.append(output, klipper, rrf);
    }
}

pub(super) struct ObjectLabels {
    name: String,
    object_id: u32,
    copy_id: u32,
    label_id: u32,
    encoded_labels: [u8; 12],
    exclude_start: Option<String>,
    exclude_end: Option<String>,
}

impl ObjectLabels {
    pub(super) fn from_traversal(
        traversal: &PreparedPostClassicTraversal,
        object_index: usize,
        copy_index: usize,
        inventory: &LabelInventory,
    ) -> Option<Self> {
        let (resolved, group) = traversal.print_object_config(object_index);
        let object = &traversal.project.objects()[resolved.source_object_index];
        let instance = &object.instances()[group.instance_indices[copy_index]];
        let encoded_labels = inventory.encode_one(instance.labeled_id())?;
        let (exclude_start, exclude_end) =
            super::footprint::in_print_labels(traversal, object_index, copy_index);
        // Beta GCode.cpp:4336-4340 assigns label IDs before the exclusion gate.
        Some(Self {
            name: object.name().to_owned(),
            object_id: object_index as u32,
            copy_id: copy_index as u32,
            label_id: instance.labeled_id(),
            encoded_labels,
            exclude_start,
            exclude_end,
        })
    }

    pub(super) fn append_printing(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            format!(
                "; printing object {} id:{} copy {}\n",
                self.name, self.object_id, self.copy_id,
            )
            .as_bytes(),
        );
    }

    pub(super) const fn start_label_data(&self) -> (u32, [u8; 12]) {
        (self.label_id, self.encoded_labels)
    }

    pub(super) const fn exclude_start(&self) -> Option<&String> {
        self.exclude_start.as_ref()
    }

    pub(super) const fn exclude_end(&self) -> Option<&String> {
        self.exclude_end.as_ref()
    }

    /// Arms the in-print object-start marker: BBL queues the M624 label
    /// (`GCode.cpp:2583-2602, 5356-5359`); Klipper/Marlin queue
    /// `EXCLUDE_OBJECT_START` / `M486 S<n>` (`GCode.cpp:5361-5372`), flushed
    /// at the first travel (`GCode.cpp:7467`). The `; printing object`
    /// comment is gated by gcode_label_objects separately
    /// (`GCode.cpp:5348-5352`).
    pub(super) fn queue_start(
        &self,
        output: &mut Vec<u8>,
        state: &mut super::motion::EmitState,
        emit_comments: bool,
    ) {
        if emit_comments {
            self.append_printing(output);
        }
        if state.tags.is_bbl() {
            let (label_id, encoded_labels) = self.start_label_data();
            super::motion::queue_object_start(state, label_id, encoded_labels);
        } else if let Some(start) = self.exclude_start() {
            super::motion::queue_exclude_start(state, start.clone());
        }
    }

    /// Arms the in-print object-end marker after instance content
    /// (`GCode.cpp:5478-5494`); the end label is only armed when the
    /// instance's start label was actually flushed — an unflushed start
    /// (empty wiping/normal half-pass, `GCode.cpp:5326-5329`) is dropped
    /// without arming the end. BBL writes its M625 tail directly.
    pub(super) fn queue_stop(
        &self,
        output: &mut Vec<u8>,
        state: &mut super::motion::EmitState,
        emit_comments: bool,
    ) {
        if emit_comments {
            self.append_stopping(output);
        }
        if state.tags.is_bbl() {
            super::motion::queue_object_stop_label(state, self.label_id);
        } else if let Some(end) = self.exclude_end() {
            super::motion::queue_exclude_end(state, end.clone());
        }
    }

    pub(super) fn append_stopping(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            format!(
                "; stop printing object {} id:{} copy {}\n",
                self.name, self.object_id, self.copy_id,
            )
            .as_bytes(),
        );
    }
}

fn encode_base64(bytes: [u8; 8]) -> [u8; 12] {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = [b'='; 12];
    for (chunk_index, chunk) in bytes.chunks(3).enumerate() {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        let output_index = chunk_index * 4;
        output[output_index] = ALPHABET[((value >> 18) & 63) as usize];
        output[output_index + 1] = ALPHABET[((value >> 12) & 63) as usize];
        if chunk.len() > 1 {
            output[output_index + 2] = ALPHABET[((value >> 6) & 63) as usize];
        }
        if chunk.len() > 2 {
            output[output_index + 3] = ALPHABET[(value & 63) as usize];
        }
    }
    output
}
