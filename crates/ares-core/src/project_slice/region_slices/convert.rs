use super::VolumeSlices;
use crate::project_slice::volume_bounds::PostBoundsVolume;

pub(super) fn volume(volume: PostBoundsVolume) -> VolumeSlices {
    let (_, occurrence_id, _, layers) = volume.into_parts();
    VolumeSlices {
        occurrence_id,
        layers: layers
            .into_iter()
            .map(|layer| layer.into_parts().1)
            .collect(),
    }
}
