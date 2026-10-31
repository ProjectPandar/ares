pub(super) fn check(project: &[u8]) {
    let processed = super::state_oracle::processed(project);
    crate::filament_volume_oracle::check(&processed);
}
