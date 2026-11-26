//! CORE-MATH binary64 `atan2` rational coefficients, revision
//! `3270006242ceef2885767918533ac5446807f2cf`, `src/binary64/atan2/atan2.c`.
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

use super::tint::Tint;

const fn t(h: u64, m: u64, l: u64, ex: i64, sgn: u64) -> Tint {
    Tint::new(h, m, l, ex, sgn)
}

pub(super) const P: [Tint; 30] = [
    t(
        0x82703f8b53112eca,
        0x71ce6279d467aa45,
        0x019c9b28466b6fc0,
        -10,
        0,
    ),
    t(
        0xb3cf74b427d53e03,
        0xee590b839e751890,
        0xf2570b12bcb2381a,
        -8,
        0,
    ),
    t(
        0xb08498b17b88c39a,
        0x11e37821d784d8fa,
        0x2795b14b84b9d78c,
        -6,
        0,
    ),
    t(
        0xfd4baa7bb52b0d83,
        0xb25425beb0a04dc3,
        0x92008d9fb34b68ff,
        -5,
        0,
    ),
    t(
        0x96541948b85fb386,
        0xe0b815ace425a859,
        0xbeca17c69bb4f7a0,
        -3,
        0,
    ),
    t(
        0x96aaa5d4baffc590,
        0xf23db783b6e4ce55,
        0x2c54e054d3b69f7e,
        -2,
        0,
    ),
    t(
        0x83cd41b5fa862b11,
        0xad8f9d6e3eb9d833,
        0x4ef470d801fa03ca,
        -1,
        0,
    ),
    t(
        0xcbbc7752197cc106,
        0xbc66cfb3edefa6e3,
        0x568b2f9e4feb8910,
        -1,
        0,
    ),
    t(
        0x8cf8f7c585767042,
        0xcd92a5b9ac13d8c8,
        0x043bd4a023dbe04d,
        0,
        0,
    ),
    t(
        0xafdc36d6b36139bf,
        0x11df63474e3859d4,
        0x42b30b2c089a2b6c,
        0,
        0,
    ),
    t(
        0xc6f8fb1463822519,
        0x2f9f04cf55c10377,
        0x1c44675c6d483606,
        0,
        0,
    ),
    t(
        0xcce79f10f754e315,
        0x91ce47edcbc8a4ee,
        0xb2a6f721ac656b42,
        0,
        0,
    ),
    t(
        0xc090384dbd77132e,
        0x1dfb4d9a1d595de7,
        0xe506ca899a394f70,
        0,
        0,
    ),
    t(
        0xa556ac9eee25e867,
        0xd84dafa25141d203,
        0xf1c7c2a5156b3ec3,
        0,
        0,
    ),
    t(
        0x81bd2677ddfcbc38,
        0x1df33f27dbb60939,
        0xcb4c2c73286a6c29,
        0,
        0,
    ),
    t(
        0xb9ed12169d72e8d9,
        0xc81d95dcef13c227,
        0x974b95796b93c332,
        -1,
        0,
    ),
    t(
        0xf2df6da875b666fa,
        0xc29dd3a69c274c65,
        0x83b9260976c8f0c5,
        -2,
        0,
    ),
    t(
        0x902d34d4db910ed8,
        0x6f3ecddfd6af0a9f,
        0x8e950d7e70a8e9b2,
        -2,
        0,
    ),
    t(
        0x9aefa41961ce1207,
        0xacd0317666753751,
        0x3eaf42a53fcc6534,
        -3,
        0,
    ),
    t(
        0x95db9977c5e6e263,
        0x85367a63fab8a720,
        0xfc2b63bd115998d8,
        -4,
        0,
    ),
    t(
        0x817cb6d8236d5f4d,
        0x70c7815f28909d9b,
        0x0c54e97155d12158,
        -5,
        0,
    ),
    t(
        0xc5eec4228c352098,
        0x789e56be1dee9b67,
        0xba25bda81292ccc1,
        -7,
        0,
    ),
    t(
        0x840c38ccfcb5dc64,
        0xb55a44095bf7f312,
        0x6668b8792d4ecd08,
        -8,
        0,
    ),
    t(
        0x97074af7b286b19e,
        0xc20aac21f3130f1d,
        0x369e9fdbdd48528a,
        -10,
        0,
    ),
    t(
        0x9054011c32d4b7a5,
        0xc68d3c5d525f4af4,
        0xeeb3c98615dee188,
        -12,
        0,
    ),
    t(
        0xdddb4a2eaab2bb15,
        0x753761a8f0e9aa8d,
        0x3f20620cd3e075bb,
        -15,
        0,
    ),
    t(
        0x80ef9a1b3e59abe5,
        0x18e6afe0651fdf49,
        0xd1c0329e6a702d9d,
        -17,
        0,
    ),
    t(
        0xca16dd49cd30854c,
        0xbad8ca379401027c,
        0x1fc7f47985486f16,
        -21,
        0,
    ),
    t(
        0xa0ba01b8742ba801,
        0xe6dc14a9516d7f04,
        0x1f5303d878905d2f,
        -25,
        0,
    ),
    t(
        0x9c0440896ce6515c,
        0x0cf3470fa7f10ac3,
        0xa9dc80682c111a9d,
        -59,
        1,
    ),
];
