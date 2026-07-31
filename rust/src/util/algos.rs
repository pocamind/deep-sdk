/* algo implementations */

use crate::{Stat, util::statmap::StatMap};

use std::collections::{HashMap, HashSet};

#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "values are not big enough for this to matter"
)]
pub fn shrine_order_dwb(pre: &StatMap, racial: &StatMap) -> StatMap {
    use crate::constants::SHRINE_ORDER_MAX_LOSS as SHRINE_DIFF_CAP;
    use crate::constants::STAT_CAP;

    let points_start = pre.cost();

    let mut work: HashMap<Stat, f64> = pre
        .iter()
        .map(|(stat, value)| (*stat, *value as f64))
        .collect();

    let mut total = 0.0_f64;
    let mut divide_by: i64 = 0;
    let mut affected_stats: Vec<Stat> = Vec::new();

    for (stat, value) in pre.iter() {
        if *value <= 0 {
            continue;
        }

        let racial_val = racial.get(stat);

        if racial_val > 0 && *value - racial_val <= 0 {
            continue;
        }

        total += (*value - racial_val.max(0)) as f64;
        affected_stats.push(*stat);
        divide_by += 1;
    }

    if divide_by == 0 {
        return pre.clone();
    }

    let average = total / divide_by as f64;
    for stat in &affected_stats {
        work.insert(*stat, average);
    }

    let mut bottlenecked_divide_by = divide_by;
    let mut bottlenecked: HashSet<Stat> = HashSet::new();
    let mut prev = work.clone();

    loop {
        let mut bottlenecked_points = 0.0_f64;
        let mut bottlenecked_stats = false;

        for stat in &affected_stats {
            if stat.is_attunement() {
                continue;
            }

            let prev_val = *prev.get(stat).unwrap_or(&0.0);
            let shrine_val = pre.get(stat) as f64;
            let current = *work.get(stat).unwrap_or(&0.0);

            if shrine_val - current > SHRINE_DIFF_CAP {
                let new_val = shrine_val - SHRINE_DIFF_CAP;
                work.insert(*stat, new_val);
                bottlenecked_points += new_val - prev_val;

                if bottlenecked.insert(*stat) {
                    bottlenecked_divide_by -= 1;
                }
            }
        }

        if bottlenecked_divide_by <= 0 {
            break;
        }

        let spread = bottlenecked_points / bottlenecked_divide_by as f64;

        // Second pass: redistribute
        for stat in &affected_stats {
            if bottlenecked.contains(stat) {
                continue;
            }

            let current = *work.get(stat).unwrap_or(&0.0);
            let next = current - spread;
            work.insert(*stat, next);

            if !stat.is_attunement() {
                let shrine_val = pre.get(stat) as f64;
                if shrine_val - next > SHRINE_DIFF_CAP {
                    bottlenecked_stats = true;
                }
            }
        }

        prev.clone_from(&work);

        if !bottlenecked_stats {
            break;
        }
    }

    let mut result = pre.clone();
    #[allow(
        clippy::cast_possible_truncation,
        reason = "value is floored before converting to i64"
    )]
    for (stat, value) in work {
        result.insert(stat, value.floor() as i64);
    }

    let mut spare_points = points_start - result.cost();

    while bottlenecked_divide_by > 0 && spare_points >= bottlenecked_divide_by {
        let mut changed = false;

        for stat in &affected_stats {
            if bottlenecked.contains(stat) {
                continue;
            }

            if result.get(stat) >= STAT_CAP {
                continue;
            }

            *result.entry(*stat).or_insert(0) += 1;
            spare_points -= 1;
            changed = true;
        }

        if !changed {
            break;
        }
    }

    result
}
