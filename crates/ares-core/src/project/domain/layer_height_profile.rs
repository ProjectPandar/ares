use super::ProjectObject;

impl ProjectObject {
    pub(crate) fn layer_height_profile(&self) -> &[f64] {
        &self.layer_height_profile
    }

    pub(crate) fn set_layer_height_profile(&mut self, profile: Vec<f64>) {
        self.layer_height_profile = profile;
    }
}
