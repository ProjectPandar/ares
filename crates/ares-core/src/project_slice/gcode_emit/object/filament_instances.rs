//! GCode.cpp:2586-2592,5255-5258,7331-7346: live per-tool instance masks.

pub(in crate::project_slice::gcode_emit) struct FilamentInstances {
    label_ids: Vec<u32>,
}

impl FilamentInstances {
    pub(in crate::project_slice::gcode_emit) fn new(ids: impl Iterator<Item = u32>) -> Self {
        let mut label_ids = ids.collect::<Vec<_>>();
        label_ids.sort_unstable();
        Self { label_ids }
    }

    pub(in crate::project_slice::gcode_emit) fn encode(
        &self,
        ids: impl Iterator<Item = u32>,
    ) -> [u8; 12] {
        assert!(self.label_ids.len() < 64);
        let mut bitset = 0_u64;
        for id in ids {
            let index = self.label_ids.partition_point(|&label| label < id);
            assert_eq!(self.label_ids[index], id, "unknown label object id");
            bitset |= 1_u64 << index;
        }
        assert_ne!(bitset, 0, "label object id error");
        super::encode_base64(bitset.to_le_bytes())
    }
}
