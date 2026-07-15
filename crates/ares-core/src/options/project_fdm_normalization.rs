mod stage1;
mod stage2;

use std::fmt;

use super::ProjectSettings;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectFdmNormalizationKey {
    EnablePrimeTower,
    IndependentSupportLayerHeight,
}

impl AsRef<str> for ProjectFdmNormalizationKey {
    fn as_ref(&self) -> &str {
        match self {
            Self::EnablePrimeTower => "enable_prime_tower",
            Self::IndependentSupportLayerHeight => "independent_support_layer_height",
        }
    }
}

impl fmt::Display for ProjectFdmNormalizationKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_ref())
    }
}

pub(crate) fn normalize_fdm_1(settings: &mut ProjectSettings) {
    stage1::normalize(settings);
}

pub(crate) fn normalize_fdm_2(
    settings: &mut ProjectSettings,
    num_objects: usize,
    used_filaments: usize,
) -> Vec<ProjectFdmNormalizationKey> {
    stage2::normalize(settings, num_objects, used_filaments)
}
