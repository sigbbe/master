use crate::dyft::VCodeTools;
use crate::lsh::CurveToIdx;
use get_size::GetSize;
use std::collections::HashMap;

#[derive(GetSize)]
pub struct LSHTable<T, const F: bool = false> {
    m_hash_values: Vec<(T, usize)>,           // (hash, trajectory_idx)
    m_buckets: HashMap<T, CurveToIdx<usize>>, // (hash, (start, end))
    m_max_curve_length: usize,
    m_distinct: usize,
}

impl<T: VCodeTools> LSHTable<T, false> {
    pub fn new(num_curves: usize, max_curve_length: usize) -> Self {
        LSHTable::<T, false> {
            m_hash_values: vec![(T::zero(), 0); num_curves],
            m_buckets: HashMap::with_capacity(0),
            m_max_curve_length: max_curve_length,
            m_distinct: 0,
        }
    }

    pub fn put(&mut self, index: usize, hash: T) {
        if let Some((h, idx)) = self.m_hash_values.get_mut(index) {
            *h = hash;
            *idx = index;
        }
    }

    #[allow(dead_code)]
    pub fn fix_table(mut self) -> LSHTable<T, true> {
        self.m_hash_values.sort();
        let mut distinct = 1;
        let mut begin_idx = 0;
        let (mut last, _) = self
            .m_hash_values
            .first_mut()
            .expect("LSHTable::fix_table: cannot fix an empty table!");
        let n = self.m_hash_values.len();

        for idx_i in 1..n {
            let (hash, _) = self.m_hash_values[idx_i];
            if idx_i == n - 1 || last != hash {
                distinct += 1;
                self.m_buckets.insert(last, (begin_idx, idx_i));
                begin_idx = idx_i;
                if idx_i != n - 1 {
                    last = hash;
                }
            }
        }

        LSHTable::<T, true> {
            m_hash_values: self.m_hash_values,
            m_buckets: self.m_buckets,
            m_max_curve_length: self.m_max_curve_length,
            m_distinct: distinct,
        }
    }

    #[allow(dead_code)]
    pub fn new_fix_table(mut self) -> LSHTable<T, true> {
        let mut distinct = 1;
        let mut begin_idx = 0;
        let (mut last, _) = self
            .m_hash_values
            .first_mut()
            .expect("LSHTable::fix_table: cannot fix an empty table!");
        let n = self.m_hash_values.len();

        self.m_hash_values.shrink_to_fit();
        self.m_hash_values.sort_unstable();
        let m_buckets: HashMap<T, CurveToIdx<usize>> = self
            .m_hash_values
            .iter()
            .skip(1)
            .zip(1..n)
            .filter_map(|(&(hash, _), idx)| {
                if idx != n - 1 {
                    if last != hash {
                        let ret = Some((last, (begin_idx, idx)));
                        distinct += 1;
                        last = hash;
                        begin_idx = idx;
                        ret
                    } else {
                        Some((last, (begin_idx, idx)))
                    }
                } else {
                    None
                }
            })
            .collect();

        LSHTable::<T, true> {
            m_buckets,
            m_hash_values: self.m_hash_values,
            m_max_curve_length: self.m_max_curve_length,
            m_distinct: distinct,
        }
    }
}

impl<T: VCodeTools> LSHTable<T, true> {
    pub fn collision_iter<'a>(&'a self, hash: T) -> Option<impl Iterator<Item = usize> + 'a> {
        self.m_buckets
            .get(&hash)
            .map(|&(start, end)| (start..end).map(|idx| self.m_hash_values[idx].1))
    }

    pub fn count_colissions<'a>(&self, hash: T, counters: &'a mut [f64]) -> &'a mut [f64] {
        if let Some(collisions) = self.collision_iter(hash) {
            collisions.for_each(|idx| {
                counters[idx] += 1.0;
            });
        }
        counters
    }

    #[allow(dead_code)]
    pub fn for_each_collision(&self, hash: T, visited: &mut [bool], callback: impl Fn(usize)) {
        if let Some(collisions) = self.collision_iter(hash) {
            collisions.for_each(|idx| match visited.get_mut(idx) {
                Some(visited) => {
                    if !*visited {
                        *visited = true;
                        callback(idx);
                    }
                }
                None => {}
            });
        }
    }

    pub fn distinct_hashes(&self) -> usize {
        self.m_distinct
    }
}
