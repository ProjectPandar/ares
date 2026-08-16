use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;

pub(super) struct ObjectLabels {
    name: String,
    object_id: u32,
    copy_id: u32,
    label_id: u32,
    encoded_labels: String,
}

impl ObjectLabels {
    pub(super) fn from_traversal(traversal: &PreparedPostClassicTraversal) -> Option<Self> {
        let object = traversal.project.objects().first()?;
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
        Some(Self {
            name: object.name().to_owned(),
            object_id: object.id(),
            copy_id: instance.instance_id(),
            label_id: instance.loaded_label_id(),
            encoded_labels: encode_base64(bitset.to_le_bytes()),
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

    pub(super) fn append_start_label(&self, output: &mut Vec<u8>) {
        output.extend_from_slice(
            format!(
                "; start printing object, unique label id: {}\nM624 {}\n",
                self.label_id, self.encoded_labels,
            )
            .as_bytes(),
        );
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

fn encode_base64(bytes: [u8; 8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(12);
    for chunk in bytes.chunks(3) {
        let value = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        output.push(ALPHABET[((value >> 18) & 63) as usize] as char);
        output.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        output.push(if chunk.len() > 1 {
            ALPHABET[((value >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(value & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

#[cfg(test)]
mod tests {
    use super::encode_base64;

    #[test]
    fn sole_label_uses_orca_little_endian_bitset_encoding() {
        assert_eq!(encode_base64(1_u64.to_le_bytes()), "AQAAAAAAAAA=");
    }
}
