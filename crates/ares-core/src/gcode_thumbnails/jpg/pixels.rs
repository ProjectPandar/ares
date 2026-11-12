/*
 * jccolor.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2009-2012, 2015, 2022, 2024-2025, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains input colorspace conversion routines.
 */
/*
 * jccolext.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright (C) 2009-2012, 2015, 2022, 2025, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains input colorspace conversion routines.
 */
/*
 * jcsample.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * Lossless JPEG Modifications:
 * Copyright (C) 1999, Ken Murchison.
 * libjpeg-turbo Modifications:
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2015, 2019, 2022, 2024-2026, D. R. Commander.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains downsampling routines.
 *
 * Downsampling input data is counted in "row groups".  A row group is defined
 * to be max_v_samp_factor rows of each component, from which the downsampler
 * produces v_samp_factor sample rows.  A single row group is processed in each
 * call to the downsampler module.
 *
 * The downsampler is responsible for edge-expansion of its output data
 * to fill an integral number of DCT blocks horizontally.  The source buffer
 * may be modified if it is helpful for this purpose (the source buffer is
 * allocated wide enough to correspond to the desired output width).
 * The caller (the prep controller) is responsible for vertical padding.
 *
 * The downsampler may request "context rows" by setting need_context_rows
 * during startup.  In this case, the input arrays will contain at least one
 * row group's worth of components above and below the passed-in data; the
 * caller will create dummy rows at image top and bottom by replicating the
 * first or last real component row(s).
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 *
 * The downsampling algorithm used here is a simple average of the source
 * components covered by the output sample.  The hi-falutin sampling literature
 * refers to this as a "box filter".  In general, the characteristics of a box
 * filter are not very good.  However, for the specific cases we normally use
 * (1:1 and 2:1 ratios), the box is equivalent to a "triangle filter", which is
 * not nearly so bad.  If you intend to use other sampling ratios, you'd be
 * well advised to improve this code.
 *
 * A simple input-smoothing capability is provided.  This is mainly intended
 * for cleaning up color-dithered GIF input files.  (If you find it inadequate,
 * we suggest using an external filtering program such as pnmconvol).  When
 * enabled, each input component C is replaced by a weighted sum of itself and
 * its eight neighbors.  C's weight is 1-8*SF, and each neighbor's weight is
 * SF, where SF = (smoothing_factor / 1024).
 * Currently, smoothing is only supported for 2h2v sampling factors.
 */
// Rust port copyright (C) 2026 Ares contributors.
// Fixed libjpeg-turbo3.2.0 thumbnail path; see README.ijg.
use super::super::ThumbnailData;

pub(super) struct Plane {
    pub width: usize,
    pub height: usize,
    stride: usize,
    samples: Vec<u8>,
}

pub(super) fn planes(image: &ThumbnailData) -> [Plane; 3] {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let mut planes: [Plane; 3] = std::array::from_fn(|_| Plane {
        width,
        height,
        stride: width,
        samples: Vec::with_capacity(width * height),
    });
    for row in image.pixels().chunks_exact(width * 4).rev() {
        for rgba in row.as_chunks::<4>().0 {
            let [r, g, b, _] = rgba.map(i32::from);
            planes[0]
                .samples
                .push(((19595 * r + 38470 * g + 7471 * b + 32768) >> 16) as u8);
            planes[1]
                .samples
                .push(((-11059 * r - 21709 * g + 32768 * b + (128 << 16) + 32767) >> 16) as u8);
            planes[2]
                .samples
                .push(((32768 * r - 27439 * g - 5329 * b + (128 << 16) + 32767) >> 16) as u8);
        }
    }
    let [y, cb, cr] = planes;
    [y, cb.downsample(), cr.downsample()]
}

impl Plane {
    fn downsample(self) -> Self {
        let width = self.width.div_ceil(2);
        let height = self.height.div_ceil(2);
        let stride = width.div_ceil(8) * 8;
        let mut samples = Vec::with_capacity(stride * height);
        for y in 0..height {
            let upper = (y * 2).min(self.height - 1) * self.stride;
            let lower = (y * 2 + 1).min(self.height - 1) * self.stride;
            for x in 0..stride {
                let left = (x * 2).min(self.width - 1);
                let right = (x * 2 + 1).min(self.width - 1);
                let sum = usize::from(self.samples[upper + left])
                    + usize::from(self.samples[upper + right])
                    + usize::from(self.samples[lower + left])
                    + usize::from(self.samples[lower + right])
                    + 1
                    + (x & 1);
                samples.push((sum >> 2) as u8);
            }
        }
        Self {
            width,
            height,
            stride,
            samples,
        }
    }

    pub fn block(&self, x: usize, y: usize) -> [i32; 64] {
        std::array::from_fn(|index| {
            let row = (y * 8 + index / 8).min(self.height - 1);
            let column = (x * 8 + index % 8).min(self.stride - 1);
            i32::from(self.samples[row * self.stride + column]) - 128
        })
    }
}
