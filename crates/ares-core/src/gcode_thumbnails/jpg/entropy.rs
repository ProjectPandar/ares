/*
 * jchuff.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1997, Thomas G. Lane.
 * Lossless JPEG Modifications:
 * Copyright (C) 1999, Ken Murchison.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2009-2011, 2014-2016, 2018-2026, D. R. Commander.
 * Copyright (C) 2015, Matthieu Darbois.
 * Copyright (C) 2018, Matthias Räncker.
 * Copyright (C) 2020, Arm Limited.
 * Copyright (C) 2022, Felix Hanau.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains Huffman entropy encoding routines.
 *
 * Much of the complexity here has to do with supporting output suspension.
 * If the data destination module demands suspension, we want to be able to
 * back up to the start of the current MCU.  To do this, we copy state
 * variables into local working storage, and update them back to the
 * permanent JPEG objects only upon successful completion of an MCU.
 *
 * NOTE: All referenced figures are from
 * Recommendation ITU-T T.81 (1992) | ISO/IEC 10918-1:1994.
 */
// Rust port copyright (C) 2026 Ares contributors.
// Fixed libjpeg-turbo3.2.0 thumbnail path; see README.ijg.
use super::tables::{COUNTS, NATURAL, VALUES};

pub(super) struct Encoder<'a> {
    output: &'a mut Vec<u8>,
    codes: [[(u16, u8); 256]; 4],
    dc: [i32; 3],
    bits: u32,
    count: u8,
}

fn codebook(table: usize) -> [(u16, u8); 256] {
    let mut result = [(0, 0); 256];
    let mut code = 0_u32;
    let mut symbol = 0;
    for (length, &count) in COUNTS[table].iter().enumerate().skip(1) {
        for _ in 0..count {
            result[usize::from(VALUES[table][symbol])] = (code as u16, length as u8);
            code += 1;
            symbol += 1;
        }
        code <<= 1;
    }
    result
}

impl<'a> Encoder<'a> {
    pub fn new(output: &'a mut Vec<u8>) -> Self {
        let codes = std::array::from_fn(codebook);
        Self {
            output,
            codes,
            dc: [0; 3],
            bits: 0,
            count: 0,
        }
    }

    pub fn block(&mut self, block: &[i32; 64], component: usize) {
        let table = if component == 0 { 0 } else { 2 };
        let difference = block[0] - self.dc[component];
        self.dc[component] = block[0];
        self.value(difference, table, 0);
        let mut zeros = 0;
        for &index in &NATURAL[1..] {
            let value = block[index];
            if value == 0 {
                zeros += 1;
                continue;
            }
            while zeros >= 16 {
                self.symbol(table + 1, 0xf0);
                zeros -= 16;
            }
            self.value(value, table + 1, zeros << 4);
            zeros = 0;
        }
        if zeros != 0 {
            self.symbol(table + 1, 0);
        }
    }

    fn value(&mut self, value: i32, table: usize, prefix: usize) {
        let count = (32 - value.unsigned_abs().leading_zeros()) as u8;
        self.symbol(table, prefix | usize::from(count));
        let amplitude = if value < 0 { value - 1 } else { value };
        self.push(amplitude as u32 & ((1 << count) - 1), count);
    }

    fn symbol(&mut self, table: usize, symbol: usize) {
        let (code, count) = self.codes[table][symbol];
        self.push(u32::from(code), count);
    }

    fn push(&mut self, bits: u32, count: u8) {
        self.bits = (self.bits << count) | bits;
        self.count += count;
        while self.count >= 8 {
            self.count -= 8;
            let byte = (self.bits >> self.count) as u8;
            self.output.push(byte);
            if byte == 255 {
                self.output.push(0);
            }
        }
    }

    pub fn finish(mut self) {
        if self.count != 0 {
            let count = 8 - self.count;
            self.push((1 << count) - 1, count);
        }
    }
}
