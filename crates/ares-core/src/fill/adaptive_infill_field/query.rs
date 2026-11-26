//! `AdaptiveInfillField` query methods (`AdaptiveInfillField.cpp:491-629`).

use crate::ProcessAdaptiveInfillMode;

use super::{AZIMUTH, AdaptiveInfillField, Lobe, MAX_MORPH, POLAR, Radial, Radials};

const EPSILON: f64 = 1.0e-4;
const LOBE_MORPH: f64 = 0.1;
const SAMPLES: usize = 32;
const PI: f64 = std::f64::consts::PI;

impl AdaptiveInfillField {
    pub(crate) fn empty(&self) -> bool {
        self.bodies.is_empty()
    }

    pub(crate) fn axis(&self) -> i32 {
        self.axis
    }

    pub(crate) fn mode(&self) -> ProcessAdaptiveInfillMode {
        self.mode
    }

    pub(super) fn directions(&self) -> usize {
        if self.axis < 0 {
            POLAR * AZIMUTH
        } else {
            AZIMUTH
        }
    }

    pub(crate) fn depth(&self, pt: [f64; 3]) -> f64 {
        debug_assert!(!self.depth.is_empty());
        let mut n0 = [0_usize; 3];
        let mut w = [0.0_f64; 3];
        for a in 0..3 {
            let f = (pt[a] - self.origin[a]) / self.cell;
            let idx = (f.floor() as i32).clamp(0, self.size[a] - 2);
            n0[a] = idx as usize;
            w[a] = (f - f64::from(idx)).clamp(0.0, 1.0);
        }
        let sy = self.size[0] as usize;
        let sz = sy * self.size[1] as usize;
        let mut value = 0.0_f64;
        for c in 0..8_usize {
            let n = (n0[2] + (c >> 2)) * sz + (n0[1] + ((c >> 1) & 1)) * sy + (n0[0] + (c & 1));
            let wx = if (c & 1) != 0 { w[0] } else { 1.0 - w[0] };
            let wy = if ((c >> 1) & 1) != 0 {
                w[1]
            } else {
                1.0 - w[1]
            };
            let wz = if (c >> 2) != 0 { w[2] } else { 1.0 - w[2] };
            value += wx * wy * wz * f64::from(self.depth[n]);
        }
        value
    }

    fn offset(&self, pt: [f64; 3], center: [f64; 3]) -> [f64; 3] {
        let mut d = [pt[0] - center[0], pt[1] - center[1], pt[2] - center[2]];
        if self.axis >= 0 {
            d[self.axis as usize] = 0.0;
        }
        d
    }

    fn radial_lobe(&self, lobe: &Lobe, pt: [f64; 3]) -> f64 {
        let d = self.offset(pt, lobe.center);
        let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let warp = |mean_depth: f64| (4.0 / 3.0 * (1.0 - mean_depth)).max(0.0);
        if r < EPSILON {
            return if lobe.mean_depth.is_empty() {
                0.0
            } else {
                warp(f64::from(lobe.mean_depth[0]))
            };
        }
        let mut i = 0usize;
        let mut fi = 0.0_f64;
        let mut azimuth = if self.axis < 0 {
            let polar = ((d[2] / r).clamp(-1.0, 1.0).acos() / PI * POLAR as f64 - 0.5)
                .clamp(0.0, (POLAR - 1) as f64);
            i = (polar as usize).min(POLAR - 2);
            fi = polar - i as f64;
            d[1].atan2(d[0])
        } else {
            let a = self.axis as usize;
            d[(a + 2) % 3].atan2(d[(a + 1) % 3])
        };
        azimuth *= AZIMUTH as f64 / (2.0 * PI);
        if azimuth < 0.0 {
            azimuth += AZIMUTH as f64;
        }
        let j0 = (azimuth as usize) % AZIMUTH;
        let j1 = (j0 + 1) % AZIMUTH;
        let fj = azimuth - azimuth.floor();
        let at = |row: usize, col: usize| f64::from(lobe.reach[row * AZIMUTH + col]);
        let mut reach = at(i, j0) * (1.0 - fj) + at(i, j1) * fj;
        if fi > 0.0 {
            reach = reach * (1.0 - fi) + (at(i + 1, j0) * (1.0 - fj) + at(i + 1, j1) * fj) * fi;
        }
        let tau = r / reach;
        if lobe.mean_depth.is_empty() {
            return tau;
        }
        let mean_at = |row: usize, col: usize| -> f64 {
            let base = (row * AZIMUTH + col) * (SAMPLES + 1);
            if tau >= 1.0 {
                return f64::from(lobe.mean_depth[base + SAMPLES]) / (tau * tau * tau);
            }
            let x = tau * SAMPLES as f64;
            let k = (x as usize).min(SAMPLES - 1);
            f64::from(lobe.mean_depth[base + k])
                + (f64::from(lobe.mean_depth[base + k + 1]) - f64::from(lobe.mean_depth[base + k]))
                    * (x - k as f64)
        };
        let mut mean = mean_at(i, j0) * (1.0 - fj) + mean_at(i, j1) * fj;
        if fi > 0.0 {
            mean = mean * (1.0 - fi)
                + (mean_at(i + 1, j0) * (1.0 - fj) + mean_at(i + 1, j1) * fj) * fi;
        }
        warp(mean)
    }

    pub(crate) fn radial(&self, pt: [f64; 3], out: &mut Radials) -> usize {
        debug_assert!(!self.empty());
        let mut idx = [0_i32; 3];
        for axis in 0..3 {
            idx[axis] = (((pt[axis] - self.origin[axis]) / self.cell).round() as i32)
                .clamp(0, self.size[axis] - 1);
        }
        let node = |idx: [i32; 3]| -> usize {
            (idx[2] as usize * self.size[1] as usize + idx[1] as usize) * self.size[0] as usize
                + idx[0] as usize
        };
        if self.axis < 0 {
            return self.body_radial(node(idx), pt, 1.0, out);
        }
        let a = self.axis as usize;
        let f = ((pt[a] - self.origin[a]) / self.cell).clamp(0.0, (self.size[a] - 1) as f64);
        let k = (f as usize).min(self.size[a] as usize - 2);
        let w = (f - k as f64) as f32;
        let mut count = 0usize;
        if w < 1.0 {
            idx[a] = self.section[k];
            count += self.body_radial(node(idx), pt, 1.0 - w, &mut out[count..]);
        }
        if w > 0.0 {
            idx[a] = self.section[k + 1];
            count += self.body_radial(node(idx), pt, w, &mut out[count..]);
        }
        count
    }

    fn body_radial(&self, node: usize, pt: [f64; 3], weight: f32, out: &mut [Radial]) -> usize {
        let body = &self.bodies[self.body[node] as usize];
        if body.lobes == 1 {
            let lobe = &self.lobes[body.first_lobe];
            out[0] = Radial {
                center: lobe.center,
                t: self.radial_lobe(lobe, pt),
                weight,
            };
            return 1;
        }
        let mut nearest = [(0.0_f64, 0usize); MAX_MORPH];
        let mut count = 0usize;
        for l in body.first_lobe..body.first_lobe + body.lobes {
            let off = self.offset(pt, self.lobes[l].center);
            let d =
                (off[0] * off[0] + off[1] * off[1] + off[2] * off[2]).sqrt() / self.lobes[l].depth;
            if count < MAX_MORPH {
                nearest[count] = (d, l);
                count += 1;
            } else if d < nearest[MAX_MORPH - 1].0 {
                nearest[MAX_MORPH - 1] = (d, l);
            } else {
                continue;
            }
            let mut k = count - 1;
            while k > 0 && nearest[k].0 < nearest[k - 1].0 {
                nearest.swap(k, k - 1);
                k -= 1;
            }
        }
        let mut blend = [0.0_f64; MAX_MORPH];
        let mut total = 0.0_f64;
        let mut morphs = 0usize;
        while morphs < count {
            let u = 0.5 - (nearest[morphs].0 - nearest[0].0) / LOBE_MORPH;
            if u <= 0.0 {
                break;
            }
            blend[morphs] = u * u * (3.0 - 2.0 * u);
            total += blend[morphs];
            morphs += 1;
        }
        for k in 0..morphs {
            let lobe = &self.lobes[nearest[k].1];
            out[k] = Radial {
                center: lobe.center,
                t: self.radial_lobe(lobe, pt),
                weight: (f64::from(weight) * blend[k] / total) as f32,
            };
        }
        morphs
    }
}
