use super::VolumeSlices;

impl VolumeSlices {
    pub(in crate::project_slice) const fn source_volume_index(&self) -> usize {
        self.source_volume_index
    }

    pub(in crate::project_slice) fn first_layer(&self) -> &[crate::geometry::ExPolygon] {
        self.layers.first().map_or(&[], Vec::as_slice)
    }

    pub(in crate::project_slice) fn volume_type(&self) -> crate::ProjectVolumeType {
        self.volume_type
    }
}

impl super::PostRegion {
    pub(in crate::project_slice) fn as_parts(
        &self,
    ) -> (usize, &crate::RegionOptions, &[super::RegionLayer]) {
        (self.id, &self.options, &self.layers)
    }

    #[cfg(test)]
    pub(in crate::project_slice) fn into_parts(
        self,
    ) -> (usize, crate::RegionOptions, Vec<super::RegionLayer>) {
        (self.id, self.options, self.layers)
    }
}
