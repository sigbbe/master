use super::table::LSHTable;
use crate::dyft::VCodeTools;
use crate::lsh::CurveToIdx;
use crate::lsh::MultiTrajectoryLsh;
use crate::lsh::TensoredMultiHash;
use crate::lsh::TrajectoryLsh;
use crate::point::Distance;
use crate::trajectory::Trajectory;
use get_size::GetSize;
use itertools::Itertools;
use rand::Fill;

pub struct MultiTable<H, const F: bool = false>
where
    H: TrajectoryLsh,
    H::Hash: VCodeTools,
{
    m_tables: Vec<LSHTable<H::Hash, F>>,
    m_max_curve_length: usize,
    m_size: usize,
}

impl<H> MultiTable<H>
where
    H: TrajectoryLsh,
    H::Hash: VCodeTools + Send + Sync,
    [H::Hash]: Fill,
{
    pub fn new(tables: Vec<LSHTable<H::Hash>>, max_curve_length: usize) -> Self {
        MultiTable {
            m_size: 0,
            m_tables: tables,
            m_max_curve_length: max_curve_length,
        }
    }

    pub fn put(&mut self, hasher: &TensoredMultiHash<H>, dataset: &[Trajectory]) {
        // let mut m_tables = self.m_tables;
        self.m_size += dataset.len();
        dataset
            .iter()
            .enumerate()
            .for_each(|(trajectory_id, trajectory)| {
                self.m_tables
                    .iter_mut()
                    .zip(hasher.multi_hash(trajectory))
                    .for_each(|(table, hash)| table.put(trajectory_id, hash));
            });
    }

    pub fn fix_table(self) -> MultiTable<H, true> {
        MultiTable {
            m_tables: self
                .m_tables
                .into_iter()
                .map(|table| table.fix_table())
                .collect(),
            m_max_curve_length: self.m_max_curve_length,
            m_size: self.m_size,
        }
    }
}

impl<H> MultiTable<H, true>
where
    H: TrajectoryLsh,
    H::Hash: VCodeTools,
    [H::Hash]: Fill,
{
    pub fn query_iter<'a>(
        &'a self,
        hasher: &'a TensoredMultiHash<H>,
        queryset: &'a [Trajectory],
    ) -> impl Iterator<Item = CurveToIdx<usize>> + 'a {
        queryset
            .iter()
            .enumerate()
            .flat_map(move |(qid, query)| {
                self.collision_iter(hasher, query)
                    .map(move |did| (qid, did))
            })
            .unique()
    }

    pub fn query_count_scores<'a>(
        &'a self,
        hasher: &'a TensoredMultiHash<H>,
        queryset: &'a [Trajectory],
    ) -> impl Iterator<Item = (Distance, CurveToIdx<usize>)> + 'a {
        let mut scores = Vec::with_capacity(self.m_size);
        let mut counters = vec![0f64; self.m_size];
        queryset.iter().enumerate().for_each(|(qid, query)| {
            scores.extend(
                self.count_collisions(hasher, query, &mut counters)
                    .map(move |(tid, collisions)| (*collisions, (qid, tid))),
            );
        });
        scores.into_iter()
    }

    fn collision_iter<'a>(
        &'a self,
        hasher: &'a TensoredMultiHash<H>,
        query: &'a Trajectory,
    ) -> impl Iterator<Item = usize> + 'a {
        self.m_tables
            .iter()
            .zip(hasher.multi_hash(query))
            .filter_map(|(table, hash)| table.collision_iter(hash))
            .flatten()
    }

    /// Count collisions for a given query.
    /// Returns an iterator over the number of collisions for each data point.
    /// The iterator is sorted by data point index.
    fn count_collisions<'a>(
        &'a self,
        hasher: &TensoredMultiHash<H>,
        query: &Trajectory,
        counters: &'a mut [f64],
    ) -> impl Iterator<Item = (usize, &'a f64)> + 'a {
        counters.fill(0.0);
        self.m_tables
            .iter()
            .zip(hasher.multi_hash(query))
            .fold(counters, |acc, (table, hash)| {
                table.count_colissions(hash, acc)
            })
            .iter()
            .enumerate()
            .filter(|(_, &count)| count > 0.0)
    }

    pub fn distinct_hashes<V>(&self) -> V
    where
        V: FromIterator<usize>,
    {
        self.m_tables
            .iter()
            .map(|table| table.distinct_hashes())
            .collect()
    }
}

impl<H, const B: bool> MultiTable<H, B>
where
    H: TrajectoryLsh,
{
    pub fn size(&self) -> usize {
        self.m_size
    }
}

impl<H, const B: bool> GetSize for MultiTable<H, B>
where
    H: TrajectoryLsh,
{
    fn get_heap_size(&self) -> usize {
        self.m_tables
            .iter()
            .map(|table| table.get_heap_size())
            .sum::<usize>()
            + self.m_max_curve_length.get_heap_size()
            + self.m_size.get_heap_size()
    }
}
