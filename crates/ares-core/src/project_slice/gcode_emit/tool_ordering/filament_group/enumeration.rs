//! Final2.5 FilamentGroup.cpp:245–414,472–495,571–623,1313–1329.
use super::{context::Context, sequence};
use std::collections::BTreeSet;

mod manual;
pub(super) use manual::solve as solve_manual;

#[derive(Clone)]
struct Remembered {
    labels: Vec<usize>,
    score: f64,
    preference: usize,
}

impl Remembered {
    fn worse_than(&self, other: &Self) -> bool {
        self.preference < other.preference
            || (self.preference == other.preference && self.score > other.score)
    }
}

// std::priority_queue<MemoryedGroup, vector<MemoryedGroup>, greater<MemoryedGroup>>.
// Preserve libstdc++ push/pop equal-key traversal rather than adding a map/serial tie-break.
#[derive(Default)]
struct Memory(Vec<Remembered>);

impl Memory {
    fn push(&mut self, item: Remembered) {
        self.0.push(item);
        let mut hole = self.0.len() - 1;
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if !self.0[parent].worse_than(&self.0[hole]) {
                break;
            }
            self.0.swap(parent, hole);
            hole = parent;
        }
    }

    fn pop(&mut self) -> Option<Remembered> {
        let value = self.0.pop()?;
        if self.0.is_empty() {
            return Some(value);
        }
        let result = std::mem::replace(&mut self.0[0], value);
        let value = self.0[0].clone();
        let mut hole = 0;
        while 2 * hole + 1 < self.0.len() {
            let left = 2 * hole + 1;
            let right = left + 1;
            let child = if right < self.0.len() && !self.0[right].worse_than(&self.0[left]) {
                right
            } else {
                left
            };
            self.0[hole] = self.0[child].clone();
            hole = child;
        }
        self.0[hole] = value;
        while hole > 0 {
            let parent = (hole - 1) / 2;
            if !self.0[parent].worse_than(&self.0[hole]) {
                break;
            }
            self.0.swap(parent, hole);
            hole = parent;
        }
        Some(result)
    }

    fn remember(&mut self, item: Remembered, one_change_budget: f64) {
        let Some(best) = self.0.first() else {
            self.push(item);
            return;
        };
        if best.preference > item.preference {
            return;
        }
        if best.preference < item.preference {
            self.0.clear();
            self.push(item);
            return;
        }
        let acceptable = |candidate: &Remembered, best: &Remembered| {
            let tolerance = (10.0_f64 * 6.0)
                .max(one_change_budget * 5.0)
                .max(best.score * 0.01);
            (candidate.score - best.score).abs() <= tolerance
        };
        if best.score <= item.score {
            if acceptable(&item, best) {
                self.push(item);
            }
            return;
        }
        let mut replacement = Self::default();
        replacement.push(item.clone());
        while let Some(old) = self.pop() {
            if acceptable(&old, &item) {
                replacement.push(old);
            }
        }
        *self = replacement;
    }
}

fn nozzle_hash(volume: i32, head: usize) -> u64 {
    let mut hash = 14_695_981_039_346_656_037_u64;
    for value in [
        volume as u64 + 0xa5a5_a5a5_a5a5_a5a5,
        head as u64 + 0x5a5a_5a5a_5a5a_5a5a,
    ] {
        hash ^= value;
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

fn calc_one_change_budget(context: &Context) -> f64 {
    let n = context.used.len();
    if n < 2 {
        return 0.0;
    }
    let mut pairs = Vec::with_capacity(n * (n - 1));
    for matrix in &context.matrices {
        for (index, &from) in context.used.iter().enumerate() {
            for &to in &context.used[index + 1..] {
                pairs.push(matrix[from][to].max(matrix[to][from]));
            }
        }
    }
    let middle = pairs.len() / 2;
    let (_, median, _) = pairs.select_nth_unstable_by(middle, f32::total_cmp);
    f64::from(*median) * sequence::FLUSH_VOLUME_TO_SCORE * 1.2
}

pub(super) fn solve(context: &Context) -> Vec<usize> {
    let one_change_budget = calc_one_change_budget(context);
    let mut seen = BTreeSet::new();
    let mut memory = Memory::default();
    let mut best_score = f64::MAX;
    let mut best_preference = 0;
    let mut primary = vec![context.master; context.total];
    let hashes = std::array::from_fn::<_, 2, _>(|head| nozzle_hash(context.volumes[head], head));
    for mask in 0..(1_usize << context.used.len()) {
        let labels = (0..context.used.len())
            .map(|index| (mask >> index) & 1)
            .collect::<Vec<_>>();
        let mut occupancy = [0; 2];
        let mut bits = [0_u64; 2];
        let mut placeable = 0;
        let mut full = vec![context.master; context.total];
        for (index, (&id, &head)) in context.used.iter().zip(&labels).enumerate() {
            occupancy[head] += 1;
            bits[head] |= 1 << index;
            full[id] = head;
            if !context.limits[head].contains(&id)
                && !context.volume_limits[id].contains(&context.volumes[head])
            {
                placeable += 1;
            }
        }
        let mut key = (0..2)
            .filter(|head| occupancy[*head] > 0)
            .map(|head| {
                let h = hashes[head];
                h ^ bits[head]
                    .wrapping_add(0x9e37_79b9)
                    .wrapping_add(h << 6)
                    .wrapping_add(h >> 2)
            })
            .collect::<Vec<_>>();
        key.sort_unstable();
        if !seen.insert(key) {
            continue;
        }
        // BestCost, with source-default ModelOnly usage_type: support preference adds zero.
        let preference = placeable * 10000
            + if (0..2).all(|head| occupancy[head] <= context.capacity[head]) {
                5000
            } else {
                0
            };
        let mut score = sequence::score(context, &full);
        if occupancy[context.master] < context.used.len().div_ceil(2) {
            score += 10.0;
        }
        if preference > best_preference || (preference == best_preference && score < best_score) {
            best_score = score;
            best_preference = preference;
            primary = full;
        }
        memory.remember(
            Remembered {
                labels,
                score,
                preference,
            },
            one_change_budget,
        );
    }
    select_best_group_for_ams(context, primary, memory)
}

fn select_best_group_for_ams(
    context: &Context,
    primary: Vec<usize>,
    mut memory: Memory,
) -> Vec<usize> {
    let remembered = std::iter::from_fn(|| memory.pop()).map(|candidate| {
        let mut map = vec![0; context.total];
        for (&id, head) in context.used.iter().zip(candidate.labels) {
            map[id] = head;
        }
        map
    });
    // Final select_best_group_for_ams: synthetic CLI filaments have no tray_name,
    // so inventory is empty. Keep the first map without materializing the others.
    std::iter::once(primary)
        .chain(remembered)
        .next()
        .expect("primary grouping candidate")
}
