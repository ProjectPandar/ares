use super::{ProjectInstance, Transform3d};

impl ProjectInstance {
    pub(crate) fn new(
        identity: [u32; 3],
        printable: bool,
        auto_drop: bool,
        transform: Transform3d,
    ) -> Self {
        Self {
            object_id: identity[0],
            instance_id: identity[1],
            loaded_label_id: Some(identity[2]),
            printable,
            auto_drop,
            transform,
        }
    }

    pub(crate) fn unassigned(
        object_id: u32,
        instance_id: u32,
        printable: bool,
        auto_drop: bool,
        transform: Transform3d,
    ) -> Self {
        Self {
            object_id,
            instance_id,
            loaded_label_id: None,
            printable,
            auto_drop,
            transform,
        }
    }

    pub fn object_id(&self) -> u32 {
        self.object_id
    }

    pub fn instance_id(&self) -> u32 {
        self.instance_id
    }

    pub fn loaded_label_id(&self) -> Option<u32> {
        self.loaded_label_id
    }

    pub fn printable(&self) -> bool {
        self.printable
    }

    pub fn auto_drop(&self) -> bool {
        self.auto_drop
    }

    pub fn transform(&self) -> Transform3d {
        self.transform
    }
}
