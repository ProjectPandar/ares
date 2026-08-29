use super::max_during_print_speed;
use crate::{OrcaBool, OrcaInt};

#[test]
fn speed_uses_only_extruders_with_both_activation_flags() {
    let activate = [OrcaBool(true), OrcaBool(true), OrcaBool(false)];
    let during = [OrcaBool(true), OrcaBool(false), OrcaBool(true)];
    let speeds = [OrcaInt(40), OrcaInt(100), OrcaInt(80)];

    assert_eq!(
        max_during_print_speed(&activate, &during, &speeds),
        Some(40)
    );
}

#[test]
fn speed_reuses_last_vector_value_and_takes_the_maximum() {
    let activate = [OrcaBool(true)];
    let during = [OrcaBool(true), OrcaBool(true)];
    let speeds = [OrcaInt(40), OrcaInt(80)];

    assert_eq!(
        max_during_print_speed(&activate, &during, &speeds),
        Some(80)
    );
}
