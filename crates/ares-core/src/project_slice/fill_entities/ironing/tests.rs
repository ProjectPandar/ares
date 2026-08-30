use super::enabled;
use crate::{ProcessIroningType, RegionOptions};

#[test]
fn all_ironing_types_use_source_layer_gates() {
    let mut region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    region.top_shell_layers = crate::OrcaInt(3);
    region.bottom_shell_layers = crate::OrcaInt(2);

    assert!(!enabled(
        ProcessIroningType::NoIroning,
        &region,
        true,
        false
    ));
    assert!(enabled(ProcessIroningType::Solid, &region, false, false));
    assert!(enabled(ProcessIroningType::Top, &region, false, false));
    assert!(!enabled(ProcessIroningType::Topmost, &region, false, false));
    assert!(enabled(ProcessIroningType::Topmost, &region, true, false));

    region.top_shell_layers = crate::OrcaInt(0);
    assert!(!enabled(ProcessIroningType::Top, &region, false, false));
    assert!(enabled(ProcessIroningType::Top, &region, false, true));
}
