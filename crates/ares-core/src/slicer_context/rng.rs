const SIZE: usize = 624;
const MIDDLE: usize = 397;

#[derive(Debug)]
pub(crate) struct Mt19937 {
    state: [u32; SIZE],
    index: usize,
}

impl Default for Mt19937 {
    fn default() -> Self {
        // std::mersenne_twister_engine::default_seed.
        Self::new(5489)
    }
}

impl Mt19937 {
    pub(crate) fn new(seed: u32) -> Self {
        let mut state = [0; SIZE];
        state[0] = seed;
        for index in 1..SIZE {
            let previous = state[index - 1];
            state[index] = 1_812_433_253_u32
                .wrapping_mul(previous ^ (previous >> 30))
                .wrapping_add(index as u32);
        }
        Self { state, index: SIZE }
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        if self.index == SIZE {
            for index in 0..SIZE {
                let word = (self.state[index] & 0x8000_0000)
                    | (self.state[(index + 1) % SIZE] & 0x7fff_ffff);
                self.state[index] = self.state[(index + MIDDLE) % SIZE]
                    ^ (word >> 1)
                    ^ (0x9908_b0df & 0_u32.wrapping_sub(word & 1));
            }
            self.index = 0;
        }
        let mut word = self.state[self.index];
        self.index += 1;
        word ^= word >> 11;
        word ^= (word << 7) & 0x9d2c_5680;
        word ^= (word << 15) & 0xefc6_0000;
        word ^ (word >> 18)
    }

    /// libstdc++15 bits/uniform_int_dist.h:255-359, int32 over mt19937.
    pub(crate) fn integer(&mut self, minimum: i32, maximum: i32) -> i32 {
        let range = (maximum as u32).wrapping_sub(minimum as u32);
        let offset = if range == u32::MAX {
            self.next_u32()
        } else {
            let width = range + 1;
            let mut product = u64::from(self.next_u32()) * u64::from(width);
            if (product as u32) < width {
                let threshold = width.wrapping_neg() % width;
                while (product as u32) < threshold {
                    product = u64::from(self.next_u32()) * u64::from(width);
                }
            }
            (product >> 32) as u32
        };
        (minimum as u32).wrapping_add(offset) as i32
    }

    pub(crate) fn unit(&mut self) -> f64 {
        let low = f64::from(self.next_u32());
        let high = f64::from(self.next_u32());
        let value = (low + high * 4_294_967_296.0) / 18_446_744_073_709_551_616.0;
        if value >= 1.0 {
            f64::from_bits(1.0_f64.to_bits() - 1)
        } else {
            value
        }
    }
}
