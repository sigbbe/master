use super::VCodeArray;
use super::VCodeTools;
use serde::Serialize;

/// https://github.com/kampersanda/dyft/blob/master/include/dyft_interface.hpp
pub trait DyFT<T: VCodeTools> {
    fn build(&mut self, m_database: &VCodeArray<T>, n: usize);

    fn append(&mut self, vcode: &[T], database: &VCodeArray<T>);

    fn trie_search(&self, vcode: &[T]) -> impl Iterator<Item = u32>;
}

pub trait DyFTStatistics {
    fn stats(&self) -> DyFTStats;

    fn size(&self) -> usize;

    fn leaves(&self) -> usize;

    fn depth(&self) -> usize;

    fn split_count(&self) -> usize;

    fn innode_stats(&self) -> impl Iterator<Item = InNodeStats>;
}

#[derive(Debug, Serialize)]
pub struct DyFTStats {
    pub(crate) size: usize,
    pub(crate) leaves: usize,
    pub(crate) depth: usize,
    pub(crate) split_count: usize,
    pub(crate) innode_stats: Vec<InNodeStats>,
    pub(crate) partial_verification_count: usize,
    pub(crate) full_verification_count: usize,
    pub(crate) filtered_verification_count: usize,
}

impl DyFTStats {
    pub fn partial_verification_count(&mut self, count: usize) {
        self.partial_verification_count = count;
    }

    pub fn full_verification_count(&mut self, count: usize) {
        self.full_verification_count = count;
    }

    pub fn filtered_verification_count(&mut self, count: usize) {
        self.filtered_verification_count = count;
    }
}

pub trait PopulationStatistics {
    fn population_stats(&self) -> PopulationStats;
}

#[derive(Debug, Serialize)]
pub struct PopulationStats {
    pub(crate) k: usize,
    pub(crate) sum: usize,
    pub(crate) nodes: Vec<usize>,
}

pub trait InNodeStatistics {
    fn innode_stats(&self) -> InNodeStats;
}

#[derive(Debug, Serialize)]
pub struct InNodeStats {
    pub(crate) k: usize,
    pub(crate) num: usize,
    pub(crate) empty: usize,
}
