//! CORE-MATH binary64 sin tables, revision 3270006242ceef2885767918533ac5446807f2cf, src/binary64/sin/sin.c.
/* Correctly-rounded sine function for binary64 value.

Copyright (c) 2026 Paul Zimmermann and Alexei Sibidanov

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

/// 1280-bit approximation of `1 / (2 * pi)` for Payne-Hanek reduction.
pub(super) const T: [u64; 20] = [
    0x0000000000000000,
    0x28be60db9391054a,
    0x7f09d5f47d4d3770,
    0x36d8a5664f10e410,
    0x7f9458eaf7aef158,
    0x6dc91b8e909374b8,
    0x01924bba82746487,
    0x3f877ac72c4a69cf,
    0xba208d7d4baed121,
    0x3a671c09ad17df90,
    0x4e64758e60d4ce7d,
    0x272117e2ef7e4a0e,
    0xc7fe25fff7816603,
    0xfbcbc462d6829b47,
    0xdb4d9fb3c9f2c26d,
    0xd3d18fd9a797fa8b,
    0x5d49eeb1faf97c5e,
    0xcf41ce7de294a4ba,
    0x9afed7ec47e35742,
    0x1580cc11bf1edaea,
];

/// Quarter-wave `sin(j * pi / 128)` double-double pairs (`0..=64`), reconstructing `U1[128][4]` via symmetry.
pub(super) const U1_SIN: [[u64; 2]; 65] = [
    [0x0000000000000000, 0x0000000000000000],
    [0x3f992155f7a3667e, 0xbbfb1d63091a0130],
    [0x3fa91f65f10dd814, 0xbc2912bd0d569a90],
    [0x3fb2d52092ce19f6, 0xbc49a088a8bf6b2c],
    [0x3fb917a6bc29b42c, 0xbc3e2718d26ed688],
    [0x3fbf564e56a9730e, 0x3c4a2704729ae56d],
    [0x3fc2c8106e8e613a, 0x3c513000a89a11e0],
    [0x3fc5e214448b3fc6, 0x3c6531ff779ddac6],
    [0x3fc8f8b83c69a60b, 0xbc626d19b9ff8d82],
    [0x3fcc0b826a7e4f63, 0xbc1af1439e521935],
    [0x3fcf19f97b215f1b, 0xbc642deef11da2c4],
    [0x3fd111d262b1f677, 0x3c7824c20ab7aa9a],
    [0x3fd294062ed59f06, 0xbc75d28da2c4612d],
    [0x3fd4135c94176601, 0x3c70c97c4afa2518],
    [0x3fd58f9a75ab1fdd, 0xbc1efdc0d58cf620],
    [0x3fd7088530fa459f, 0xbc744b19e0864c5d],
    [0x3fd87de2a6aea963, 0xbc672cedd3d5a610],
    [0x3fd9ef7943a8ed8a, 0x3c66da81290bdbab],
    [0x3fdb5d1009e15cc0, 0x3c65b362cb974183],
    [0x3fdcc66e9931c45e, 0x3c56850e59c37f8f],
    [0x3fde2b5d3806f63b, 0x3c5e0d891d3c6841],
    [0x3fdf8ba4dbf89aba, 0xbc32ec1fc1b776b8],
    [0x3fe073879922ffee, 0xbc8a5a014347406c],
    [0x3fe11eb3541b4b23, 0xbc8ef23b69abe4f1],
    [0x3fe1c73b39ae68c8, 0x3c8b25dd267f6600],
    [0x3fe26d054cdd12df, 0xbc85da743ef3770c],
    [0x3fe30ff7fce17035, 0xbc6efcc626f74a6f],
    [0x3fe3affa292050b9, 0x3c7e3e25e3954964],
    [0x3fe44cf325091dd6, 0x3c68076a2cfdc6b3],
    [0x3fe4e6cabbe3e5e9, 0x3c63c293edceb327],
    [0x3fe57d69348ceca0, 0xbc875720992bfbb2],
    [0x3fe610b7551d2cdf, 0xbc7251b352ff2a37],
    [0x3fe6a09e667f3bcd, 0xbc8bdd3413b26456],
    [0x3fe72d0837efff96, 0x3c80d4ef0f1d915c],
    [0x3fe7b5df226aafaf, 0xbc70f537acdf0ad7],
    [0x3fe83b0e0bff976e, 0xbc76f420f8ea3475],
    [0x3fe8bc806b151741, 0xbc82c5e12ed1336d],
    [0x3fe93a22499263fb, 0x3c83d419a920df0b],
    [0x3fe9b3e047f38741, 0xbc830ee286712474],
    [0x3fea29a7a0462782, 0xbc7128bb015df175],
    [0x3fea9b66290ea1a3, 0x3c39f630e8b6dac8],
    [0x3feb090a58150200, 0xbc8926da300ffcce],
    [0x3feb728345196e3e, 0xbc8bc69f324e6d61],
    [0x3febd7c0ac6f952a, 0xbc8825a732ac700a],
    [0x3fec38b2f180bdb1, 0xbc76e0b1757c8d07],
    [0x3fec954b213411f5, 0xbc52fb761e946603],
    [0x3feced7af43cc773, 0xbc5e7b6bb5ab58ae],
    [0x3fed4134d14dc93a, 0xbc84ef5295d25af2],
    [0x3fed906bcf328d46, 0x3c7457e610231ac2],
    [0x3feddb13b6ccc23c, 0x3c883c37c6107db3],
    [0x3fee212104f686e5, 0xbc8014c76c126527],
    [0x3fee6288ec48e112, 0xbc616b56f2847754],
    [0x3fee9f4156c62dda, 0x3c8760b1e2e3f81e],
    [0x3feed740e7684963, 0x3c7e82c791f59cc2],
    [0x3fef0a7efb9230d7, 0x3c752c7adc6b4989],
    [0x3fef38f3ac64e589, 0xbc7d7bafb51f72e6],
    [0x3fef6297cff75cb0, 0x3c7562172a361fd3],
    [0x3fef8764fa714ba9, 0x3c7ab256778ffcb6],
    [0x3fefa7557f08a517, 0xbc87a0a8ca13571f],
    [0x3fefc26470e19fd3, 0x3c81ec8668ecacee],
    [0x3fefd88da3d12526, 0xbc887df6378811c7],
    [0x3fefe9cdad01883a, 0x3c6521ecd0c67e35],
    [0x3feff621e3796d7e, 0xbc6c57bc2e24aa15],
    [0x3feffd886084cd0d, 0xbc81354d4556e4cb],
    [0x3ff0000000000000, 0x0000000000000000],
];
