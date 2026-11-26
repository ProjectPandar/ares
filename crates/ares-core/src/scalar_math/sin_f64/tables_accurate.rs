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

/// Odd degree-7 fixed-point polynomial for `sin(2 * pi / 8 * x)` on `[0, 2^-11)`.
pub(super) const PS: [u128; 4] = [
    0xc90fdaa22168c234c4c6628b80dbed80,
    0x14abbce625be52beabb8e5506ca45b03,
    0x00a335e33bad570e044c54995bb97635,
    0x0000000000000000000265a5999d06c4,
];

/// Even degree-8 fixed-point polynomial for `cos(2 * pi / 8 * x)` on `[0, 2^-11)`.
pub(super) const PC: [u128; 5] = [
    0xffffffffffffffffffffffffffffffff,
    0x4ef4f326f91779692b71366cbcf0b096,
    0x040f07c206d6b0ecd4c9076e3db78fed,
    0x00155d3c7e3cbef9eec6962368fb7c41,
    0x000000000000000000003c3e845b5510,
];

/// 128-bit fixed-point `sin(pi * i / 2^6)` (`0..=32`).
pub(super) const S1U: [u128; 33] = [
    0x00000000000000000000000000000000,
    0x0c8fb2f886ec09f376a17954b2b7c517,
    0x1917a6bc29b42be1d8e72d912977ee71,
    0x259020dd1cc27444c002a2684781f080,
    0x31f17078d34c156c9732300393f33614,
    0x3e33f2f642be355e90887712e9dc9663,
    0x4a5018bb567c16a2d725d3b9ed35fbaa,
    0x563e69d6ac7f73f8408fca9cc277fc1f,
    0x61f78a9abaa58b4698916152cf7eee1c,
    0x6d744027857300ad9b165cba0c171818,
    0x78ad74e01bd8ec78362474f1a105878f,
    0x839c3cc917ff6cb4bfd79717f2880abf,
    0x8e39d9cd73464364bba4cfecbff54867,
    0x987fbfe70b81a70819cec845ac87a5c6,
    0xa267992848eeb0c03b5167ee359a234e,
    0xabeb49a46764fd151becda8089c1a94c,
    0xb504f333f9de6484597d89b3754abe9f,
    0xbdaef913557d76f0ac85320f528d6d5d,
    0xc5e40358a8ba05a743da25d99267326b,
    0xcd9f023f9c3a059e23af31db7179a4aa,
    0xd4db3148750d1819f630e8b6dac83e69,
    0xdb941a28cb71ec872c19b63253da43fc,
    0xe1c5978c05ed8691f4e8a8372f8c5810,
    0xe76bd7a1e63b9786125129529d48a92f,
    0xec835e79946a31457e610231ac1d6181,
    0xf1090827b43725fd67127db35b287316,
    0xf4fa0ab6316ed2ec163c5c7f03b718c5,
    0xf853f7dc9186b952c7adc6b4988891bb,
    0xfb14be7fbae581562172a361fd2a722f,
    0xfd3aabf84528b50beae6bd951c1dabbe,
    0xfec46d1e89292cf041390efdc726e9ef,
    0xffb10f1bcb6bef1d421e8edaaf59453e,
    0xffffffffffffffffffffffffffffffff,
];

/// 128-bit fixed-point `sin(pi * i / 2^12)` (`0..64`).
pub(super) const S2U: [u128; 64] = [
    0x00000000000000000000000000000000,
    0x003243f655d966c0c3b0abf2b35a62f8,
    0x006487eabb991cb611ad1d7b5620834a,
    0x0096cbdb41258434c4a32c4560d01e70,
    0x00c90fc5f66525d257480f7956b64707,
    0x00fb53a8eb3ec385328421cf8014d698,
    0x012d97822f996bc4f96857b5aa8f6208,
    0x015fdb4fd35c8caad230a30592f25049,
    0x01921f0fe6700711ab967fe6b7a9b037,
    0x01c462c078bc41b67db6e69ab68f0d10,
    0x01f6a65f9a2a3c5885cede1f0314607b,
    0x0228e9eb5aa3a2d97c11ebc1b33cafb5,
    0x025b2d61ca12e05dc2dd9c015a46e767,
    0x028d70c0f863326c8e8d6151676ae94d,
    0x02bfb406f580bc10053205a54588f0ab,
    0x02f1f731d15898f5556febf4862da970,
    0x03243a3f9bd8f08cc3c75f41b6ce7aa8,
    0x03567d2e64f10929ad8a2d0c1a9e3a8c,
    0x0388bffc3c915b2280c1c97a65cf2291,
    0x03bb02a732aba3f0a8493bf9c07f7765,
    0x03ed452d5732f9506b5e1380c7e6bddb,
    0x041f878cba1bdc60bfeba221fb9d05be,
    0x0451c9c36b5c4cc30ed3c01c030008bb,
    0x04840bcf7aebdbbae97857207bc589da,
    0x04b64daef8c3bf4dafc8f71b8eb233ee,
    0x04e88f5ff4dee5622617b65f5a2a8f5a,
    0x051ad0e07f3a06dff9f89fb65be2a455,
    0x054d122ea7d3bacf356ef187634a531d,
    0x057f53487eac89779fab71e43d71e2b7,
    0x05b1942c13c6ff800a9f1c1238fd05d5,
    0x05e3d4d77727c10d8ca56cd8d54b97f0,
    0x06161548b8d59ce2a58993a76b3f4ffe,
    0x0648557de8d99f7e4e29cf6e5fed0679,
    0x067a9575173f263af1fc3edb7a0f1757,
    0x06acd52c5413f26d51b86c83422cbab4,
    0x06df14a1af683c834e68e062eb177e2d,
    0x071153d3394ec7229c28010f1c93a252,
    0x074392bf01dcf2475bcb8fd41cb1096f,
    0x0775d163192ace629ac20c033d7f5cc3,
    0x07a80fbd8f532f78b8654aa824578975,
    0x07da4dcc7473c03fb00590e675e4e556,
    0x080c8b8dd8ad153d46f0804dae5f13ba,
    0x083ec8ffcc22bfe51db725856ff8c284,
    0x087106205efb61b6a3f67ad05a5be69e,
    0x08a342eda160bf5aede5b1068d174bea,
    0x08d57f65a37fd3c26aed92d34c17df54,
    0x0907bb867588e3427c8c5732d89e910b,
    0x0939f74e27af8eb2ecc93966728e412d,
    0x096c32baca2ae68b437b2dd49d5fca3c,
    0x099e6dca6d357dfff9a60c93317892c4,
    0x09d0a87b210d7e1f8a318ba775fd7b04,
    0x0a02e2caf5f4b8ef5f3d645e787b94f2,
    0x0a351cb7fc30bc889b56007d16d4ad5a,
    0x0a675640440ae634bdcd0d6bb4b9cd3e,
    0x0a998f61ddd0758a217954ed6093c44c,
    0x0acbc81ad9d29f8855213c653bfb79b7,
    0x0afe00694866a1b44cd34d2751c2e1da,
    0x0b30384b39e5d5346b7029d39efd7682,
    0x0b626fbebeadc1ec63a95642f565102f,
    0x0b94a6c1e7203198efb8391d83d6da18,
    0x0bc6dd52c3a342eb5f10bfca3d646401,
    0x0bf9136f64a17ca4f9530f050886d756,
    0x0c2b4915da89e0b235bfac0f965612e2,
    0x0c5d7e4435cfff45c6718c1dfd2aa612,
];

/// 128-bit fixed-point `cos(pi * i / 2^12)` (`0..64`).
pub(super) const C2U: [u128; 64] = [
    0xffffffffffffffffffffffffffffffff,
    0xfffffb10b0d19f76491a703231e0a12e,
    0xffffec42c3773235ec9e2e75cb525f2c,
    0xffffd3963882d5536276b75b91de105e,
    0xffffb10b10e80e953031437d7eccb9df,
    0xffff84a14dfbcc6a858425d8b397dee6,
    0xffff4e58f17465de177338053fd93920,
    0xffff0e31fd699a853a11d60588d8b96e,
    0xfffec42c7454926b38e310779edfec68,
    0xfffe7048590fddf8edd8e1034213f22b,
    0xfffe1285aed775d896f351efc65556cf,
    0xfffdaae47948bad5ea80aedd6a19710f,
    0xfffd3964bc6275ba69fff9ae0dedb047,
    0xfffcbe067c84d725f3a703b987eca44a,
    0xfffc38c9be717763928db07a0e70ba36,
    0xfffba9ae874b563a8d800bed6653dcba,
    0xfffb10b4dc96dabbb47903f7a19f8ee2,
    0xfffa6ddcc439d30aecc7b9244a48eb19,
    0xfff9c126447b7424fbe18032d0016082,
    0xfff90a91640459a190e2d2eaf6da4d1e,
    0xfff84a1e29de85718cc193c5d508e13f,
    0xfff77fcc9d755f9989332d07a713477e,
    0xfff6ab9cc695b5e89e4938f661aa140c,
    0xfff5cd8ead6dbbab66c785e86dfbb75f,
    0xfff4e5a25a8d095b43366df666fd54ff,
    0xfff3f3d7d6e49c49dbb49f29fa872a83,
    0xfff2f82f2bc6d648e08b96133ecce0bd,
    0xfff1f2a862e77d4e098a31bcda3def20,
    0xfff0e343865bbb135428ed0647c9e5d1,
    0xffefca00a09a1cb3807b6e7a4a723dae,
    0xffeea6dfbc7a9242ccf344c647917821,
    0xffed79e0e5366e63f0f7cb05bde024f1,
    0xffec4304266865d95657552366961732,
    0xffeb02498c0c8f129195e99fbca2f10a,
    0xffe9b7b1228061b6191df31aaa6f7f45,
    0xffe8633af682b6273b57790bf77b6b2d,
    0xffe704e71533c50853aa9423bb0adc21,
    0xffe59cb58c1526b93e71f7d99688082a,
    0xffe42aa66909d2d20be28fbec7cfb8a6,
    0xffe2aeb9ba561f99f1ed54343fe7be24,
    0xffe128ef8e9fc17a7d209f32d42d864e,
    0xffdf9947f4edca6f008e6ee05573d420,
    0xffddffc2fca8a97044bd28b8d85b530a,
    0xffdc5c60b59a29dc75a8951fc304b914,
    0xffdaaf212fed72db4fd8f038449ec436,
    0xffd8f8047c2f06be8c9611f0b1d8f023,
    0xffd7370aab4cc25e8d3cd437dc7fa9d2,
    0xffd56c33ce95dc7345bd035edb0a65f5,
    0xffd3977ff7bae4e9664649b4d541b9c5,
    0xffd1b8ef38cdc433c42aac754bedcfde,
    0xffcfd081a441ba9901fd552bf146b4a6,
    0xffcdde374ceb5f7d76f487bb853a6989,
    0xffcbe2104600a0a95595ca3f421ae09c,
    0xffc9dc0ca318c18b11b369083a7a62e5,
    0xffc7cc2c782c5a7605c2a6019679e41f,
    0xffc5b26fd99557dd579207cfe424dcb7,
    0xffc38ed6dc0ef98b1c676208aa3be545,
    0xffc1616194b5d1d3bc8d54e81d94f831,
    0xffbf2a101907c4c5965827f33d906c7c,
    0xffbce8e27ee40754e0aa07fcb29eef39,
    0xffba9dd8dc8b1e83ccfed60a91097c48,
    0xffb848f3489ede86e907d9a298ab1feb,
    0xffb5ea31da2269e5bfdfce09aea7ac02,
    0xffb38194a87a3097badfe70a1ef51116,
];
