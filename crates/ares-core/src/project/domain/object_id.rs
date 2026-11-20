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

    pub(crate) fn cli_stl_merge(count: u32) -> Self {
        // CLI::run reads each file into a model, then moves it to the vector.
        // Model move construction allocates the wipe tower before assignment.
        let mut read = Self::model_read();
        read.add_object();
        read.add_volume();
        read.add_instance();
        // Read return and vector insertion each move the model. Reallocation
        // copies previously loaded models; each copy also constructs a wipe tower.
        let mut copied = 0;
        let mut capacity = 1;
        while capacity < count {
            copied += capacity;
            capacity *= 2;
        }
        // The merged Model owns an ID and wipe tower (OrcaSlicer.cpp:4040–4055).
        Self((read.0 + 2) * count + copied + 2)
    }

    pub(crate) fn clone_stl_instance(&mut self) -> u32 {
        // ModelObject::assign_new_unique_ids_recursive (Model.cpp:1346–1354).
        self.add_object();
        self.add_volume();
        let instance = self.add_instance();
        self.0 += 1; // layer height profile
        instance
    }
}
