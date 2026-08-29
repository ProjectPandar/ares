use super::{flush_speed, flush_temperature};
use crate::{Nullable, OrcaFloat, OrcaFloats, OrcaInt, OrcaInts};

#[test]
fn configured_flush_values_override_fallbacks() {
    assert_eq!(
        flush_speed(
            &Nullable::Value(OrcaFloat(18.0)),
            &OrcaFloats(vec![OrcaFloat(15.0)]),
            0,
        ),
        18.0
    );
    assert_eq!(
        flush_temperature(
            &Nullable::Value(OrcaInt(230)),
            &OrcaInts(vec![OrcaInt(250)]),
            0,
        ),
        230
    );
}

#[test]
fn zero_flush_values_use_indexed_fallbacks() {
    assert_eq!(
        flush_speed(
            &Nullable::Value(OrcaFloat(0.0)),
            &OrcaFloats(vec![OrcaFloat(15.0), OrcaFloat(20.0)]),
            1,
        ),
        20.0
    );
    assert_eq!(
        flush_temperature(
            &Nullable::Value(OrcaInt(0)),
            &OrcaInts(vec![OrcaInt(250), OrcaInt(260)]),
            1,
        ),
        260
    );
}
