//! glibc `rand()`/`random()` TYPE_3 replication for byte-parity with
//! OrcaSlicer's `FillLightning` (`TreeNode.cpp:365` picks the first child in
//! `convertToPolylines` via `rand()`). Orca never calls `srand` on the normal
//! slicing path, so `rand()` yields the glibc default (seed 1) sequence.
//!
//! glibc TYPE_3: `rand_deg=31`, `rand_sep=3`, int32 state table `r[0..31)`.
//! `random()` computes `result = *fptr + *rptr; *fptr = result;` then returns
//! `(result >> 1) & 0x7fffffff`, advancing `fptr`/`rptr` with wrap-around at
//! `end_ptr = r + DEG`. `srandom(seed)` fills `r` via Schrage
//! `r[i]=16807*r[i-1] mod 2147483647`, then discards 10*DEG outputs.

const DEG: usize = 31;
const SEP: usize = 3;

pub(crate) struct GlibcRand {
    r: [i32; DEG],
    fptr: usize,
    rptr: usize,
}

impl GlibcRand {
    /// `srandom(1)` — the unseeded glibc default.
    pub(crate) fn new() -> Self {
        Self::with_seed(1)
    }

    pub(crate) fn with_seed(seed: i32) -> Self {
        let mut r = [0i32; DEG];
        r[0] = if seed == 0 { 1 } else { seed };
        // r[i] = (16807 * r[i-1]) % 2147483647, computed via Schrage to avoid
        // int32 overflow.
        for i in 1..DEG {
            let prev = r[i - 1] as i64;
            let hi = prev / 127773;
            let lo = prev % 127773;
            let mut word = 16807 * lo - 2836 * hi;
            if word < 0 {
                word += 2147483647;
            }
            r[i] = word as i32;
        }
        let mut g = Self {
            r,
            fptr: SEP,
            rptr: 0,
        };
        // Discard 10*DEG warmup outputs.
        for _ in 0..10 * DEG {
            g.next();
        }
        g
    }

    /// One glibc `random()`/`rand()` value.
    pub(crate) fn next(&mut self) -> i32 {
        let result = self.r[self.fptr].wrapping_add(self.r[self.rptr]);
        self.r[self.fptr] = result;
        let out = ((result as u32) >> 1) as i32;
        // Advance both pointers; wrap at end_ptr = r + DEG (fptr wraps to
        // r+SEP? No — both wrap to the table start; fptr and rptr both index
        // into r[0..DEG) and wrap to 0).
        self.fptr += 1;
        if self.fptr >= DEG {
            self.fptr = 0;
        }
        self.rptr += 1;
        if self.rptr >= DEG {
            self.rptr = 0;
        }
        out
    }
}
