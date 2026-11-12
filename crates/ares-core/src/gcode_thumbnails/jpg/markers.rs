/*
 * jcmarker.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1998, Thomas G. Lane.
 * Modified 2003-2010 by Guido Vollbeding.
 * Lossless JPEG Modifications:
 * Copyright (C) 1999, Ken Murchison.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2010, 2022, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains routines to write JPEG datastream markers.
 */
// Rust port copyright (C) 2026 Ares contributors.
// Fixed libjpeg-turbo3.2.0 thumbnail path; see README.ijg.
use super::tables::{COUNTS, NATURAL, VALUES};

pub(super) fn header(output: &mut Vec<u8>, width: u32, height: u32, quantization: &[[u8; 64]; 2]) {
    output.extend_from_slice(&[255, 216]);
    marker(output, 0xe0, b"JFIF\0\x01\x01\0\0\x01\0\x01\0\0");
    for (index, table) in quantization.iter().enumerate() {
        let mut payload = [0; 65];
        payload[0] = index as u8;
        for (position, &natural) in NATURAL.iter().enumerate() {
            payload[position + 1] = table[natural];
        }
        marker(output, 0xdb, &payload);
    }
    let [hh, hl] = (height as u16).to_be_bytes();
    let [wh, wl] = (width as u16).to_be_bytes();
    marker(
        output,
        0xc0,
        &[8, hh, hl, wh, wl, 3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1],
    );
    for (index, tag) in [0, 0x10, 1, 0x11].into_iter().enumerate() {
        output.extend_from_slice(&[255, 0xc4]);
        output.extend_from_slice(&((19 + VALUES[index].len()) as u16).to_be_bytes());
        output.push(tag);
        output.extend_from_slice(&COUNTS[index][1..]);
        output.extend_from_slice(VALUES[index]);
    }
    marker(output, 0xda, &[3, 1, 0, 2, 0x11, 3, 0x11, 0, 63, 0]);
}

fn marker(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.extend_from_slice(&[255, tag]);
    output.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    output.extend_from_slice(payload);
}
