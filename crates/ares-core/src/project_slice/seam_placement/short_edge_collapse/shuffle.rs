//! `std::mt19937_64{27644437}` (ShortEdgeCollapse.cpp:82) plus the exact
//! libstdc++15 `std::shuffle` traversal: the pairwise-extraction fast path
//! (`bits/stl_algo.h:3734-3767`, one `uniform_int_distribution{0,1}` swap up
//! front when the element count is even, then `__gen_two_uniform_ints` per
//! element pair, `bits/stl_algo.h:3685-3703`) and the per-element fallback
//! (`bits/stl_algo.h:3769-3770`).

pub(super) struct ShuffleRng {
    state: [u64; 312],
    index: usize,
}

impl ShuffleRng {
    pub(super) fn new(seed: u64) -> Self {
        let mut state = [0_u64; 312];
        state[0] = seed;
        for index in 1..312 {
            state[index] = 6_364_136_223_846_793_005_u64
                .wrapping_mul(state[index - 1] ^ (state[index - 1] >> 62))
                .wrapping_add(index as u64);
        }
        Self { state, index: 312 }
    }

    fn next_u64(&mut self) -> u64 {
        if self.index == 312 {
            self.twist();
        }
        let mut value = self.state[self.index];
        self.index += 1;
        value ^= (value >> 29) & 0x5555_5555_5555_5555;
        value ^= (value << 17) & 0x71d6_7fff_eda6_0000;
        value ^= (value << 37) & 0xfff7_eee0_0000_0000;
        value ^ (value >> 43)
    }

    fn twist(&mut self) {
        for index in 0..312 {
            let joined = (self.state[index] & 0xFFFF_FFFF_8000_0000)
                | (self.state[(index + 1) % 312] & 0x7FFF_FFFF);
            let matrix = if joined & 1 == 0 {
                0
            } else {
                0xB502_6F5A_A966_19E9
            };
            self.state[index] = self.state[(index + 156) % 312] ^ (joined >> 1) ^ matrix;
        }
        self.index = 0;
    }

    /// `_S_nd<unsigned __int128>` — Lemire's nearly divisionless downscale,
    /// `bits/uniform_int_dist.h:252-278` (the mt19937_64 path selected at
    /// `bits/uniform_int_dist.h:307-314`).
    fn nd_range(&mut self, range: u64) -> u64 {
        let mut product = u128::from(self.next_u64()) * u128::from(range);
        let mut low = product as u64;
        if low < range {
            let threshold = range.wrapping_neg() % range;
            while low < threshold {
                product = u128::from(self.next_u64()) * u128::from(range);
                low = product as u64;
            }
        }
        (product >> 64) as u64
    }

    /// `std::shuffle` for a generator whose range covers the count squared
    /// (always true for mt19937_64 below 2^32 elements) plus the general
    /// fallback.
    pub(super) fn shuffle(&mut self, items: &mut [usize]) {
        let count = items.len();
        if count < 2 {
            return;
        }
        // `__urange` is the element count n itself, not n-1
        // (stl_algo.h:3731).
        let urange = count as u64;
        let urngrange = u64::MAX;
        if urngrange / urange >= urange {
            // bits/stl_algo.h:3737-3766.
            let mut index = 1;
            if urange.is_multiple_of(2) {
                let position = self.nd_range(2) as usize;
                items.swap(1, position);
                index = 2;
            }
            while index != count {
                let swap_range = (index + 1) as u64;
                let draw = self.nd_range(swap_range * (swap_range + 1));
                items.swap(index, (draw / (swap_range + 1)) as usize);
                index += 1;
                items.swap(index, (draw % (swap_range + 1)) as usize);
                index += 1;
            }
            return;
        }
        // bits/stl_algo.h:3769-3770.
        for index in 1..count {
            let position = self.nd_range((index + 1) as u64) as usize;
            items.swap(index, position);
        }
    }
}
