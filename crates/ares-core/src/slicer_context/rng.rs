const SIZE: usize = 624;
const MIDDLE: usize = 397;

pub(crate) struct Mt19937 {
    state: [u32; SIZE],
    index: usize,
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
