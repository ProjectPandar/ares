//! GCode.cpp:2585-2601,5139-5203,7318-7347: global IDs and layer skip sets.

pub(in crate::project_slice::gcode_emit) struct LabelInventory {
    ids: Vec<u32>,
}

impl LabelInventory {
    pub(in crate::project_slice::gcode_emit) fn new(ids: impl Iterator<Item = u32>) -> Self {
        let mut ids = ids.collect::<Vec<_>>();
        ids.sort_unstable();
        Self { ids }
    }

    pub(in crate::project_slice::gcode_emit) fn ids(&self) -> &[u32] {
        &self.ids
    }

    fn index(&self, id: u32) -> usize {
        let index = self.ids.partition_point(|&label| label < id);
        assert_eq!(self.ids[index], id, "unknown label object id");
        index
    }

    pub(in crate::project_slice::gcode_emit) fn encode_one(&self, id: u32) -> Option<[u8; 12]> {
        let bitset = 1_u64.checked_shl(self.index(id) as u32)?;
        Some(super::encode_base64(bitset.to_le_bytes()))
    }

    pub(in crate::project_slice::gcode_emit) fn encode(
        &self,
        ids: impl Iterator<Item = u32>,
    ) -> [u8; 12] {
        assert!(self.ids.len() < 64);
        let bitset = ids.fold(0_u64, |bits, id| bits | (1_u64 << self.index(id)));
        assert_ne!(bitset, 0, "label object id error");
        super::encode_base64(bitset.to_le_bytes())
    }

    pub(in crate::project_slice::gcode_emit) fn layer(
        &self,
        ids: impl Iterator<Item = u32>,
    ) -> Option<LayerLabels> {
        // GCode.cpp:5182-5186; calibration jobs are not exposed by this API.
        if !(2..=64).contains(&self.ids.len()) {
            return None;
        }
        let mut ids = ids.collect::<Vec<_>>();
        ids.sort_unstable();
        ids.dedup();
        if ids.is_empty() {
            return None;
        }
        let encoded = self.encode(ids.iter().copied());
        Some(LayerLabels { ids, encoded })
    }
}

pub(in crate::project_slice::gcode_emit) struct LayerLabels {
    ids: Vec<u32>,
    encoded: [u8; 12],
}

impl LayerLabels {
    pub(in crate::project_slice::gcode_emit) fn append_start(
        &self,
        output: &mut Vec<u8>,
        number: usize,
    ) {
        self.append_ids(output, &format!("; object ids of layer {number} start: "));
        output.extend_from_slice(b"M624 ");
        output.extend_from_slice(&self.encoded);
        output.push(b'\n');
    }

    pub(in crate::project_slice::gcode_emit) fn append_end(
        &self,
        output: &mut Vec<u8>,
        number: usize,
    ) {
        self.append_ids(output, &format!("; object ids of this layer{number} end: "));
        output.extend_from_slice(b"M625\n");
    }

    fn append_ids(&self, output: &mut Vec<u8>, prefix: &str) {
        output.extend_from_slice(prefix.as_bytes());
        for (index, id) in self.ids.iter().enumerate() {
            if index != 0 {
                output.push(b',');
            }
            output.extend_from_slice(id.to_string().as_bytes());
        }
        output.push(b'\n');
    }
}
