use super::VolumeSlices;
use crate::project_slice::volume_bounds::PostBoundsVolume;

pub(super) fn volume(volume: PostBoundsVolume) -> VolumeSlices {
    let (source_volume_index, occurrence_id, volume_type, layers) = volume.into_parts();
    VolumeSlices {
        source_volume_index,
        occurrence_id,
        volume_type,
        layers: layers
            .into_iter()
            .map(|layer| layer.into_parts().1)
            .collect(),
    }
}
