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
    for definition in &super::footprint::definitions(traversal) {
        definition.append(output, klipper);
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
    ) -> Option<Self> {
        let object = traversal.project.objects().get(object_index)?;
        let instance = object.instances().first()?;
        let mut labels = traversal
            .project
            .objects()
            .iter()
            .flat_map(|object| object.instances())
            .map(|instance| instance.loaded_label_id())
            .collect::<Vec<_>>();
        labels.sort_unstable();
        labels.dedup();
        let position = labels.binary_search(&instance.loaded_label_id()).ok()?;
        let bitset = 1_u64.checked_shl(position as u32)?;
        let (exclude_start, exclude_end) =
            super::footprint::in_print_labels(traversal, object_index);
        Some(Self {
            name: object.name().to_owned(),
            // `PrintObject::get_id()` is the sequential print-object index,
            // not the 3MF mesh id (`GCode.cpp:5349-5352`).
            object_id: object_index as u32,
            copy_id: instance.instance_id(),
            label_id: instance.loaded_label_id(),
            encoded_labels: encode_base64(bitset.to_le_bytes()),
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

    pub(super) fn append_stopping(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            format!(
                "; stop printing object {} id:{} copy {}\n",
                self.name, self.object_id, self.copy_id,
            )
            .as_bytes(),
        );
    }

    pub(super) fn append_stop_label(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            format!(
                "; stop printing object, unique label id: {}\nM625\n",
                self.label_id,
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

#[cfg(test)]
mod tests {
    use super::encode_base64;

    #[test]
    fn sole_label_uses_orca_little_endian_bitset_encoding() {
        assert_eq!(encode_base64(1_u64.to_le_bytes()), *b"AQAAAAAAAAA=");
    }
}
