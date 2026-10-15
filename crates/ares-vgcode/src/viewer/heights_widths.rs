use super::ViewerImpl;

impl ViewerImpl {
    /// ViewerImpl.cpp:1648-1652 (RED scaffold: not yet ported).
    pub fn set_travels_radius(&mut self, _radius: f32) {}

    /// ViewerImpl.hpp:169 (RED scaffold: not yet ported).
    pub fn get_travels_radius(&self) -> f32 {
        0.0
    }

    /// ViewerImpl.cpp:1654-1658 (RED scaffold: not yet ported).
    pub fn set_wipes_radius(&mut self, _radius: f32) {}

    /// ViewerImpl.hpp:171 (RED scaffold: not yet ported).
    pub fn get_wipes_radius(&self) -> f32 {
        0.0
    }

    /// ViewerImpl.cpp:954-968 (RED scaffold: not yet ported).
    pub fn heights_widths(&self) -> Vec<[f32; 2]> {
        vec![[0.0; 2]; self.vertices.len()]
    }
}
