use std::borrow::Borrow;

use crate::{req::Requirement, Stat, util::statmap::StatMap};


// Utility for dealing with a group of reqs
pub trait ReqVecExt {
    fn map_names<F>(&mut self, f: F) 
    where
        F: Fn(&str) -> String;
}

impl ReqVecExt for Vec<Requirement> {
    fn map_names<F>(&mut self, f: F)
    where
        F: Fn(&str) -> String,
    {
        for req in self.iter_mut() {
            req.name = req.name.as_ref().map(|name| f(&name));

            req.prereqs = req.prereqs.iter().map(|name| f(&name)).collect();
        }
    }
}

pub trait ReqIterExt {
    fn max_map(self) -> StatMap;

    fn max_total_req(self) -> i64;
}

impl<I> ReqIterExt for I
where
    I: Iterator,
    I::Item: Borrow<Requirement>, 
{
    fn max_map(self) -> StatMap {
        let mut maxes: StatMap = StatMap::new();

        for req in self {
            let req = req.borrow();

            for atom in req.atoms() {
                for &stat in &atom.stats {
                    if stat == Stat::Total {
                        continue;
                    }

                    // TODO! we cant do a trivial per-stat max here,
                    // bc of sum reqs.
                    maxes
                        .entry(stat)
                        .and_modify(|cur| *cur = (*cur).max(atom.value))
                        .or_insert(atom.value);
                }
            }
        }

        maxes
    }

    fn max_total_req(self) -> i64 {
        let mut max: i64 = 0;

        for req in self {
            let req = req.borrow();

            for atom in req.atoms() {
                if atom.stats.contains(&Stat::Total) {
                    max = max.max(atom.value);
                }
            }
        }

        max
    }
}
