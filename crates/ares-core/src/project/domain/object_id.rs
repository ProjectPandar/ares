//! Fresh import constructor contributions to `ObjectBase::s_last_id`
//! (`ObjectID.hpp:76–87`, `Model.hpp:582–588`, `Model.cpp:331`).

pub(crate) struct ObjectIdCounter(u32);

impl ObjectIdCounter {
    pub(crate) const fn model_read() -> Self {
        // Outer CLI model/wipe tower, then read_from_file model/wipe tower.
        Self(2 + 2)
    }

    pub(crate) fn add_object(&mut self) {
        self.0 += 3;
    }

    pub(crate) fn add_volume(&mut self) {
        // Volume/config/four facets, plus eight displacement facets (Model.hpp:903–935).
        self.0 += 6 + 8;
    }

    pub(crate) fn add_instance(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }
}
