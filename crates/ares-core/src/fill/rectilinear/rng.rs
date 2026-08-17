const STATE_SIZE: usize = 312;
const MIDDLE_WORD: usize = 156;
const MATRIX: u64 = 0xB502_6F5A_A966_19E9;
const LOWER_MASK: u64 = (1_u64 << 31) - 1;
const UPPER_MASK: u64 = !LOWER_MASK;

pub(super) struct Mt19937_64 {
    state: [u64; STATE_SIZE],
    index: usize,
}

impl Default for Mt19937_64 {
    fn default() -> Self {
        let mut state = [0_u64; STATE_SIZE];
        state[0] = 5489;
        for index in 1..STATE_SIZE {
            let previous = state[index - 1];
            state[index] = 6_364_136_223_846_793_005_u64
                .wrapping_mul(previous ^ (previous >> 62))
                .wrapping_add(index as u64);
        }
        Self {
            state,
            index: STATE_SIZE,
        }
    }
}

impl Mt19937_64 {
    pub(super) fn next(&mut self) -> u64 {
        if self.index == STATE_SIZE {
            self.twist();
        }
        let mut value = self.state[self.index];
        self.index += 1;
        value ^= (value >> 29) & 0x5555_5555_5555_5555;
        value ^= (value << 17) & 0x71D6_7FFF_EDA6_0000;
        value ^= (value << 37) & 0xFFF7_EEE0_0000_0000;
        value ^ (value >> 43)
    }

    pub(super) fn index(&mut self, upper_exclusive: usize) -> usize {
        let range = upper_exclusive as u64;
        loop {
            let product = u128::from(self.next()) * u128::from(range);
            let low = product as u64;
            let threshold = range.wrapping_neg() % range;
            if low >= range || low >= threshold {
                return (product >> 64) as usize;
            }
        }
    }

    pub(super) fn unit_f32(&mut self) -> f32 {
        self.next() as f32 / u64::MAX as f32
    }

    fn twist(&mut self) {
        for index in 0..STATE_SIZE {
            let value = (self.state[index] & UPPER_MASK)
                | (self.state[(index + 1) % STATE_SIZE] & LOWER_MASK);
            self.state[index] = self.state[(index + MIDDLE_WORD) % STATE_SIZE]
                ^ (value >> 1)
                ^ if value & 1 == 0 { 0 } else { MATRIX };
        }
        self.index = 0;
    }
}
