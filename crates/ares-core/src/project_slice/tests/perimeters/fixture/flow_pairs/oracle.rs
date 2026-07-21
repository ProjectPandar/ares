use super::{Bits, bits};

pub(super) const INITIAL_ABSOLUTE_050_NOZZLE_04: Bits = bits(
    [0x3f000000, 0x3e4ccccd, 0x3eea0658, 0x3ecccccd],
    false,
    0x3fb76708c0000000,
);
pub(super) const ABSOLUTE_045_NOZZLE_04: Bits = bits(
    [0x3ee66666, 0x3e4ccccd, 0x3ed06cbe, 0x3ecccccd],
    false,
    0x3fb4d7aca0000000,
);
pub(super) const ABSOLUTE_042_NOZZLE_04: Bits = bits(
    [0x3ed70a3d, 0x3e4ccccd, 0x3ec11094, 0x3ecccccd],
    false,
    0x3fb34e7540000000,
);
pub(super) const PERCENT_100_NOZZLE_04: Bits = bits(
    [0x3ecccccd, 0x3e4ccccd, 0x3eb6d324, 0x3ecccccd],
    false,
    0x3fb2485080000000,
);
pub(super) const ABSOLUTE_045_NOZZLE_06: Bits = bits(
    [0x3ee66666, 0x3e4ccccd, 0x3ed06cbe, 0x3f19999a],
    false,
    0x3fb4d7aca0000000,
);
pub(super) const ABSOLUTE_042_NOZZLE_06: Bits = bits(
    [0x3ed70a3d, 0x3e4ccccd, 0x3ec11094, 0x3f19999a],
    false,
    0x3fb34e7540000000,
);
pub(super) const PERCENT_100_NOZZLE_06: Bits = bits(
    [0x3f19999a, 0x3e4ccccd, 0x3f0e9cc6, 0x3f19999a],
    false,
    0x3fbc85c120000000,
);
pub(super) const PERCENT_110_NOZZLE_06: Bits = bits(
    [0x3f28f5c3, 0x3e4ccccd, 0x3f1df8ef, 0x3f19999a],
    false,
    0x3fbf982fc0000000,
);
pub(super) const PERCENT_125_NOZZLE_06: Bits = bits(
    [0x3f400000, 0x3e4ccccd, 0x3f35032c, 0x3f19999a],
    false,
    0x3fc219eac0000000,
);
pub(super) const PERCENT_080_NOZZLE_06: Bits = bits(
    [0x3ef5c290, 0x3e4ccccd, 0x3edfc8e8, 0x3f19999a],
    false,
    0x3fb660e400000000,
);
pub(super) const PERCENT_110_NOZZLE_04: Bits = bits(
    [0x3ee147ae, 0x3e4ccccd, 0x3ecb4e06, 0x3ecccccd],
    false,
    0x3fb4549a20000000,
);
pub(super) const PERCENT_080_NOZZLE_04: Bits = bits(
    [0x3ea3d70a, 0x3e4ccccd, 0x3e8ddd62, 0x3ecccccd],
    false,
    0x3fac5f79e0000000,
);
pub(super) const OBJECT_ABSOLUTE_052_NOZZLE_04: Bits = bits(
    [0x3f051eb8, 0x3e4ccccd, 0x3ef443c8, 0x3ecccccd],
    false,
    0x3fb86d2da0000000,
);
pub(super) const NONTHICK_GROW: Bits = bits(
    [0x3ed59710, 0x3e8f5c2a, 0x3eb6d324, 0x3ecccccd],
    false,
    0x3fb99870c0000000,
);
pub(super) const NONTHICK_SHRINK: Bits = bits(
    [0x3ea83c2c, 0x3e4ccccd, 0x3e924284, 0x3ecccccd],
    false,
    0x3fad4080c0000000,
);
pub(super) const NONTHICK_ROUND: Bits = bits(
    [0x3d8a1779, 0x3d8a1779, 0x3eb6d324, 0x3ecccccd],
    false,
    0x3f6d4080a0000000,
);
pub(super) const NONTHICK_AUTO_WIDTH: Bits = bits(
    [0x3ebcb70d, 0x3e4ccccd, 0x3ea6bd64, 0x3ecccccd],
    false,
    0x3fb0ac8a00000000,
);
pub(super) const NONTHICK_PERCENT_120_RATIO_144: Bits = bits(
    [0x3eff6ddb, 0x3e9374bc, 0x3edfc8e8, 0x3ecccccd],
    false,
    0x3fc01ccd20000000,
);
pub(super) const NONTHICK_AUTO_RATIO_064: Bits = bits(
    [0x3e9b5df8, 0x3e4ccccd, 0x3e856450, 0x3ecccccd],
    false,
    0x3faaada980000000,
);
pub(super) const THICK_PERCENT_120_RATIO_144: Bits = bits(
    [0x3f1374bd, 0x3f1374bd, 0x3f20418a, 0x3ecccccd],
    true,
    0x3fd0ad4840000000,
);
pub(super) const THICK_AUTO_RATIO_064: Bits = bits(
    [0x3ea3d70b, 0x3ea3d70b, 0x3ebd70a5, 0x3ecccccd],
    true,
    0x3fb496b7e0000000,
);
