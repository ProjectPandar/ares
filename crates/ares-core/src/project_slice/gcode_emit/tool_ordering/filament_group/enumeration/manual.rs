//! FilamentGroup.cpp:199–374,427–512,901–922,1316–1345,1495–1505.
use super::{Context, Memory, Remembered, nozzle_hash, sequence};
use crate::SliceError;
use std::collections::{BTreeMap, BTreeSet};

pub(in super::super) fn solve(context: &Context) -> Result<Vec<usize>, SliceError> {
    let k = context.nozzle_groups.len();
    // The source hashes group_id > 0, not the owning extruder index.
    let hashes = context
        .nozzle_groups
        .iter()
        .map(|nozzle| nozzle_hash(nozzle.volume, usize::from(nozzle.group_id > 0)))
        .collect::<Vec<_>>();
    let total = enumeration_count(context.used.len(), &hashes)?;
    let limits = nozzle_limits(context);
    let mut seen = BTreeSet::new();
    let mut memory = Memory::default();
    let mut best_score = f64::MAX;
    let mut best_preference = 0;
    let mut primary = vec![context.master; context.total];
    for mask in 0..total {
        let mut num = mask;
        let mut labels = Vec::with_capacity(context.used.len());
        let mut occupancy = vec![0; k];
        let mut bits = vec![0_u64; k];
        let mut full = vec![context.master; context.total];
        let mut placeable = 0;
        for (index, &id) in context.used.iter().enumerate() {
            let nozzle = num % k;
            num /= k;
            labels.push(nozzle);
            occupancy[nozzle] += 1;
            bits[nozzle] |= 1 << index;
            full[id] = nozzle;
            placeable += usize::from(!limits[index].contains(&nozzle));
        }
        let mut key = (0..k)
            .filter(|&nozzle| occupancy[nozzle] > 0)
            .map(|nozzle| {
                let h = hashes[nozzle];
                h ^ bits[nozzle]
                    .wrapping_add(0x9e37_79b9)
                    .wrapping_add(h << 6)
                    .wrapping_add(h >> 2)
            })
            .collect::<Vec<_>>();
        key.sort_unstable();
        if !seen.insert(key) {
            continue;
        }
        // Manual dispatch removes capacity limits; source usage is ModelOnly/BestCost.
        let preference = placeable * 10000 + 5000;
        let mut score = sequence::score_multi_nozzle(context, &full);
        if occupancy[context.master] < context.used.len().div_ceil(2) {
            score += 10.0;
        }
        if preference > best_preference || (preference == best_preference && score < best_score) {
            best_score = score;
            best_preference = preference;
            primary = full;
        }
        memory.remember(Remembered {
            labels,
            score,
            preference,
        });
    }
    // Empty usable AMS inventory: the owning-extruder cost is 2 * 9999 per used head.
    let ams_cost = |map: &[usize]| {
        let mut heads = [false; 2];
        for &id in &context.used {
            heads[context.nozzle_groups[map[id]].extruder] = true;
        }
        heads.into_iter().filter(|used| *used).count() * 2 * 9999
    };
    let mut cost = ams_cost(&primary);
    while let Some(candidate) = memory.pop() {
        let mut map = vec![0; context.total];
        for (&id, nozzle) in context.used.iter().zip(candidate.labels) {
            map[id] = nozzle;
        }
        let candidate_cost = ams_cost(&map);
        if candidate_cost < cost {
            cost = candidate_cost;
            primary = map;
        }
    }
    // ToolOrdering.cpp:1842–1853: an unsatisfiable manual extruder map is an error.
    if context
        .used
        .iter()
        .any(|&id| context.nozzle_groups[primary[id]].extruder != context.manual_extruders[id])
    {
        return Err(SliceError::InvalidInput(
            "Group error in manual mode. Please check nozzle count or regroup.".to_owned(),
        ));
    }
    Ok(primary)
}

fn enumeration_count(n: usize, hashes: &[u64]) -> Result<usize, SliceError> {
    let mut total = 1_usize;
    for _ in 0..n {
        total = total.saturating_mul(hashes.len());
        if total > 10_000_000 {
            return Err(super::super::context::unsupported());
        }
    }
    let mut types = BTreeMap::<u64, usize>::new();
    for &hash in hashes {
        *types.entry(hash).or_default() += 1;
    }
    let mut divisor = 1_usize;
    for count in types.into_values() {
        for factor in 2..=count {
            divisor = divisor.saturating_mul(factor);
        }
    }
    if total / divisor >= 10000 {
        return Err(super::super::context::unsupported());
    }
    Ok(total)
}

fn nozzle_limits(context: &Context) -> Vec<BTreeSet<usize>> {
    context
        .used
        .iter()
        .map(|&id| {
            let mut forbidden = std::array::from_fn::<_, 2, _>(|head| {
                context.limits[head].contains(&id) || head != context.manual_extruders[id]
            });
            // extract_unprintable_limit_indices removes the cross-head intersection.
            if forbidden == [true; 2] {
                forbidden = [false; 2];
            }
            let expected = context.requested_volumes[id];
            context
                .nozzle_groups
                .iter()
                .filter(|nozzle| {
                    forbidden[nozzle.extruder]
                        || (expected != 2 && expected != nozzle.volume)
                        || context.volume_limits[id].contains(&nozzle.volume)
                })
                .map(|nozzle| nozzle.group_id)
                .collect()
        })
        .collect()
}
