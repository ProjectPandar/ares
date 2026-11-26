//! CORE-MATH binary64 `atan2` tables, revision `3270006242ceef2885767918533ac5446807f2cf`,
//! `src/binary64/atan2/atan2.c`.
/* Correctly-rounded atan2 function for two binary64 values.

Copyright (c) 2024-2025 Paul Zimmermann and Alexei Sibidanov

This file is part of the CORE-MATH project
(https://core-math.gitlabpages.inria.fr/).

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

const fn f(bits: u64) -> f64 {
    f64::from_bits(bits)
}

pub(super) const PI_H: f64 = f(0x400921fb54442d18);
pub(super) const PI_L: f64 = f(0x3ca1a62633145c07);
pub(super) const PI_OVER2_H: f64 = f(0x3ff921fb54442d18);
pub(super) const PI_OVER2_L: f64 = f(0x3c91a62633145c07);

pub(super) const FINF: [[f64; 2]; 2] = [
    [f(0x3c80000000000000), f(0x3fe921fb54442d18)],
    [f(0x3c90000000000000), f(0x4002d97c7f3321d2)],
];

pub(super) const B: [f64; 3] = [
    f(0xbfd55555555554d2),
    f(0x3fc999999860e1ca),
    f(0xbfc248ad469844a1),
];

pub(super) const B2: [f64; 4] = [
    f(0xbfd5555555555555),
    f(0x3fc999999999755e),
    f(0xbfc24924883596f8),
    f(0x3fbc6f7d73531bc2),
];

pub(super) const O: [[f64; 2]; 8] = [
    [0.0, 0.0],
    [f(0x3ff921fb54442d18), f(0x3c91a62633145c07)],
    [0.0, 0.0],
    [f(0xbff921fb54442d18), f(0xbc91a62633145c07)],
    [f(0x400921fb54442d18), f(0x3ca1a62633145c07)],
    [f(0x3ff921fb54442d18), f(0x3c91a62633145c07)],
    [f(0xc00921fb54442d18), f(0xbca1a62633145c07)],
    [f(0xbff921fb54442d18), f(0xbc91a62633145c07)],
];

pub(super) const T2: [f64; 65] = [
    0.0,
    f(0x3f90000000000000),
    f(0x3fa0000000000000),
    f(0x3fa8000000000000),
    f(0x3fb0000000000000),
    f(0x3fb4000000000000),
    f(0x3fb8000000000000),
    f(0x3fbc000000000000),
    f(0x3fc0000000000000),
    f(0x3fc2000000000000),
    f(0x3fc4000000000000),
    f(0x3fc6000000000000),
    f(0x3fc8000000000000),
    f(0x3fca000000000000),
    f(0x3fcc000000000000),
    f(0x3fce000000000000),
    f(0x3fd0000000000000),
    f(0x3fd1000000000000),
    f(0x3fd2000000000000),
    f(0x3fd3000000000000),
    f(0x3fd4000000000000),
    f(0x3fd5000000000000),
    f(0x3fd6000000000000),
    f(0x3fd7000000000000),
    f(0x3fd8000000000000),
    f(0x3fd9000000000000),
    f(0x3fda000000000000),
    f(0x3fdb000000000000),
    f(0x3fdc000000000000),
    f(0x3fdd000000000000),
    f(0x3fde000000000000),
    f(0x3fdf000000000000),
    f(0x3fe0000000000000),
    f(0x3fe0800000000000),
    f(0x3fe1000000000000),
    f(0x3fe1800000000000),
    f(0x3fe2000000000000),
    f(0x3fe2800000000000),
    f(0x3fe3000000000000),
    f(0x3fe3800000000000),
    f(0x3fe4000000000000),
    f(0x3fe4800000000000),
    f(0x3fe5000000000000),
    f(0x3fe5800000000000),
    f(0x3fe6000000000000),
    f(0x3fe6800000000000),
    f(0x3fe7000000000000),
    f(0x3fe7800000000000),
    f(0x3fe8000000000000),
    f(0x3fe8800000000000),
    f(0x3fe9000000000000),
    f(0x3fe9800000000000),
    f(0x3fea000000000000),
    f(0x3fea800000000000),
    f(0x3feb000000000000),
    f(0x3feb800000000000),
    f(0x3fec000000000000),
    f(0x3fec800000000000),
    f(0x3fed000000000000),
    f(0x3fed800000000000),
    f(0x3fee000000000000),
    f(0x3fee800000000000),
    f(0x3fef000000000000),
    f(0x3fef800000000000),
    1.0,
];

pub(super) const F2: [[f64; 2]; 65] = [
    [0.0, 0.0],
    [f(0xbca95220c39d4dff), f(0x3f8fff555bbb7300)],
    [f(0x3ca2542779d776de), f(0x3f9ffd55bba97600)],
    [f(0xbca6061bbe3de53c), f(0x3fa7fb818430da40)],
    [f(0xbca639269b0da47e), f(0x3faff55bb72cfe00)],
    [f(0xbc84a7663af440f7), f(0x3fb3f59f0e7c55a0)],
    [f(0x3cad1824d59f9e13), f(0x3fb7ee182602f100)],
    [f(0x3c8bef71e5340b31), f(0x3fbbe39ebe6f07c0)],
    [f(0x3cab8cb225e627d0), f(0x3fbfd5ba9aac2f60)],
    [f(0x3cab92de9bac94c2), f(0x3fc1e1fafb043720)],
    [f(0xbc9d3cb89e62dafd), f(0x3fc3d6eee8c66270)],
    [f(0xbca882a55960087a), f(0x3fc5c9811e3ec270)],
    [f(0x3c91347b0b4f881d), f(0x3fc7b97b4bce5b00)],
    [f(0x3ca873d8079ed0d2), f(0x3fc9a6a8e96c8620)],
    [f(0x3c9022f621a5c1cb), f(0x3fcb90d7529260a0)],
    [f(0x3ca9c648d1534598), f(0x3fcd77d5df205730)],
    [f(0xbc94ea9238610a08), f(0x3fcf5b75f92c80e0)],
    [f(0x3ca2c5c8e721970d), f(0x3fd09dc597d86360)],
    [f(0x3c630ca4748b1bf9), f(0x3fd18bf5a30bf178)],
    [f(0xbca20ef9ba6dbf90), f(0x3fd278372057ef48)],
    [f(0x3cae69c5abb498d2), f(0x3fd362773707ebc8)],
    [f(0x3c9a8a86f0ea9311), f(0x3fd44aa436c2af08)],
    [f(0x3c9db5336feef7f0), f(0x3fd530ad9951cd48)],
    [f(0x3c99636a3aa3b840), f(0x3fd614840309cfe0)],
    [f(0x3c81ce2a8c848b74), f(0x3fd6f61941e4def0)],
    [f(0xbc84b1bbd1ea6db3), f(0x3fd7d5604b63b3f8)],
    [f(0xbca4925e8b916e0b), f(0x3fd8b24d394a1b28)],
    [f(0x3c79e6c988fd0a77), f(0x3fd98cd5454d6b18)],
    [f(0xbcaa49bd836a1700), f(0x3fda64eec3cc2400)],
    [f(0xbcaca3cf09c6b5f8), f(0x3fdb3a911da65c70)],
    [f(0xbc7cc1ce70934c34), f(0x3fdc0db4c94ec9f0)],
    [f(0x3c82e982ddf3872a), f(0x3fdcde53432c1350)],
    [f(0xbc82ea406ee84d0f), f(0x3fddac670561bb50)],
    [f(0xbcade35847c81979), f(0x3fde77eb7f175a38)],
    [f(0xbc6a3992dc382a23), f(0x3fdf40dd0b541418)],
    [f(0xbc8b32c949c9d593), f(0x3fe0039c73c1a40c)],
    [f(0xbc7d5b495f6349e6), f(0x3fe0657e94db30d0)],
    [f(0xbcaf34582f6255fe), f(0x3fe0c6145b5b43dc)],
    [f(0x3c9ed42511e3f11d), f(0x3fe1255d9bfbd2a8)],
    [f(0xbc91cef189ff9e7f), f(0x3fe1835a88be7c14)],
    [f(0xbc5928df287a668f), f(0x3fe1e00babdefeb4)],
    [f(0xbcae3bde360c7ddb), f(0x3fe23b71e2cc9e6c)],
    [f(0x3c9bd86313ce4fde), f(0x3fe2958e59308e30)],
    [f(0xbca8e8a85803cc1d), f(0x3fe2ee628406cbcc)],
    [f(0xbc977ef7641c777f), f(0x3fe345f01cce37bc)],
    [f(0x3cab73ef3389d02f), f(0x3fe39c391cd41718)],
    [f(0x3c7ecf8b492644f0), f(0x3fe3f13fb89e96f4)],
    [f(0x3cac1125fd3810c7), f(0x3fe445065b795b54)],
    [f(0x3ca2483350fe548b), f(0x3fe4978fa3269ee0)],
    [f(0x3c84a33dbeb3796c), f(0x3fe4e8de5bb6ec04)],
    [f(0xbca46edd2af69483), f(0x3fe538f57b890620)],
    [f(0xbca2bcb93b18b52a), f(0x3fe587d81f732fbc)],
    [f(0x3c60028e4bc5e7ca), f(0x3fe5d58987169b18)],
    [f(0x3caed487acaf1174), f(0x3fe6220d115d7b8c)],
    [f(0xbca2dd4dfd7d1777), f(0x3fe66d663923e088)],
    [f(0x3c92bfe3cf3b9d79), f(0x3fe6b798920b3d98)],
    [f(0xbc78c34d25aadef6), f(0x3fe700a7c5784634)],
    [f(0xbc9f426acf4d3bdb), f(0x3fe748978fba8e10)],
    [f(0xbcaafe57dd9ff230), f(0x3fe78f6bbd5d3160)],
    [f(0xbc954fbef0e862ab), f(0x3fe7d528289fa094)],
    [f(0x3c990227758b11ba), f(0x3fe819d0b7158a4c)],
    [f(0x3ca16b66e7fc8b8c), f(0x3fe85d69576cc2c4)],
    [f(0xbc855b9a5e177a1b), f(0x3fe89ff5ff57f1f8)],
    [f(0x3cac27cfaa9f7a14), f(0x3fe8e17aa99cc05c)],
    [f(0x3c81a62633145c07), f(0x3fe921fb54442d18)],
];
