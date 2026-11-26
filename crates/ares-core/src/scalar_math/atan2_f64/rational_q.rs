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

pub(super) const Q: [Tint; 30] = [
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
        0xf2570b12bcb238cf,
        -8,
        0,
    ),
    t(
        0xb33c44af0d43c9e8,
        0xf4426f84614701db,
        0x979e49dc5b858442,
        -6,
        0,
    ),
    t(
        0x8264d256f0bf4d61,
        0xee21431cc0f29764,
        0xce0cc1b593ac9952,
        -4,
        0,
    ),
    t(
        0x9d97c4947687b2ee,
        0xef4087c49dde5229,
        0x742bc6e02bfb6df8,
        -3,
        0,
    ),
    t(
        0xa0f888dc7aab949e,
        0xf623f19ac413db23,
        0x854744a164f70ecd,
        -2,
        0,
    ),
    t(
        0x8fd9c7172ab0828e,
        0x1a9f72fadafc3dfb,
        0xaf96a180ea297178,
        -1,
        0,
    ),
    t(
        0xe38168b78a51ee15,
        0x604a47815c8091eb,
        0x9faaa42312e8eb30,
        -1,
        0,
    ),
    t(
        0xa164ccdbb65ee133,
        0xe623433c5e086402,
        0xcb228afd335a245f,
        0,
        0,
    ),
    t(
        0xced0c0cbe44fac2f,
        0xd7c2adfc7c5dda9e,
        0xa22b4a311cf5a5b1,
        0,
        0,
    ),
    t(
        0xf0e6660438c90371,
        0x3dca1a0f58f0d943,
        0x8a92b9de68c4488c,
        0,
        0,
    ),
    t(
        0x8000000000000000,
        0x0000000000000000,
        0x0000000000000000,
        1,
        0,
    ),
    t(
        0xf8ea68f9ad849103,
        0xb257e4bfc606967f,
        0x818fa2816928c4aa,
        0,
        0,
    ),
    t(
        0xddc64eeee3506234,
        0x47a33ea40a2b22e4,
        0xa6a2c129ace2470a,
        0,
        0,
    ),
    t(
        0xb52ddfba2d37c8e2,
        0x24551a9b19fa064a,
        0x3d78c68b54bc3850,
        0,
        0,
    ),
    t(
        0x87ac113d26b683a1,
        0x4e97756c38785b0b,
        0xf75f0a4506931ba8,
        0,
        0,
    ),
    t(
        0xba065070b900f3d5,
        0xe49d8f9f238f1eeb,
        0x6805d48d67f1dcd9,
        -1,
        0,
    ),
    t(
        0xe902d2a4bfc22441,
        0x945cfe6c83a5b828,
        0x519ebb457f8ccca0,
        -2,
        0,
    ),
    t(
        0x84e405c915f7769b,
        0x0faaf34c3c51dd39,
        0xa97fa0ae99382c93,
        -2,
        0,
    ),
    t(
        0x896da46c7e6cebb3,
        0xe0afc0b6db3dbbf5,
        0x061ccff6e78efff4,
        -3,
        0,
    ),
    t(
        0x801a0c1a0dfefc92,
        0xf8e0a97902e08d47,
        0x81898e0b7d8ef0bc,
        -4,
        0,
    ),
    t(
        0xd59d088930df07db,
        0xa8cb00021849fbaf,
        0xf80cc91aa0bddf89,
        -6,
        0,
    ),
    t(
        0x9db56a6984b77231,
        0xcf7c2074e0480959,
        0x3316ecaa6aa4604e,
        -7,
        0,
    ),
    t(
        0xcb7450d9d62d2e68,
        0x9787b4265977d004,
        0xba615cee00a841d7,
        -9,
        0,
    ),
    t(
        0xe12a52e02b64b23e,
        0x97c9139a8ced716c,
        0x70a176ad2b5f16d2,
        -11,
        0,
    ),
    t(
        0xd057b6d0900ed30d,
        0x5e04bb17eb879a5e,
        0x23ed135b60bea070,
        -13,
        0,
    ),
    t(
        0x9b1f1ca22fef7ae5,
        0x19a687f0a4073eee,
        0x05a9ccbc8c14578b,
        -15,
        0,
    ),
    t(
        0xaeba710f053aac3b,
        0x30ea5d6fb116f775,
        0x8a9bcfef5698f8f1,
        -18,
        0,
    ),
    t(
        0x84b9ada076cd007d,
        0x13e55006073f76b1,
        0x4cf4259968228f3e,
        -21,
        0,
    ),
    t(
        0xcca4b9046786937d,
        0x233c1e691c127c1f,
        0xf493e0b7a3de459a,
        -26,
        0,
    ),
];
