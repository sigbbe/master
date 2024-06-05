mod bit_tools;
mod byte_pack;
mod constants;
mod dyft;
mod io;
mod mart;
mod node;
mod sparse;

use crate::config::IndexConfig;
use crate::config::LshConfig;
use crate::config::MartConfig;
use crate::lsh::MultiTrajectoryLsh;
use crate::lsh::TensoredMultiHash;
use crate::lsh::TrajectoryLsh;
use crate::point::Distance;
use crate::trajectory::Trajectory;
use crate::util::GetIndexSize;
use crate::util::IndexSize;
use crate::util::MasterStats;
pub use bit_tools::*;
pub use byte_pack::BytePack;
pub use constants::*;
pub use dyft::*;
use get_size::GetSize;
pub use io::*;
use itertools::Itertools;
pub use mart::*;
pub use node::*;
use rand::rngs::StdRng;
use rand::Fill;
use rand::SeedableRng;
pub use sparse::SparseGroup;
pub use sparse::SparseTable;

pub type MartNodeId = u32;
pub type MartByteLabel = u8;
pub type MartPointerOffset = usize;

pub struct DyftIndex<'a, H>
where
    H: TrajectoryLsh,
    [(); H::Hash::N_DIM]:,
{
    m_hasher: TensoredMultiHash<H>,
    m_index: MartIndex<'a>,
}

impl<'a, H> DyFT<H::Hash> for DyftIndex<'a, H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
    [(); H::Hash::N_DIM]:,
{
    fn build(&mut self, vcodes: &VCodeArray<H::Hash>, n: usize) {
        self.m_index.build(vcodes, n)
    }

    fn append(&mut self, vcode: &[H::Hash], database: &VCodeArray<H::Hash>) {
        self.m_index.append(vcode, database)
    }

    fn trie_search(&self, vcode: &[H::Hash]) -> impl Iterator<Item = u32> {
        self.m_index.trie_search(vcode)
    }
}

impl<'a, H> DyFTStatistics for DyftIndex<'a, H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
    [(); H::Hash::N_DIM]:,
{
    fn stats(&self) -> DyFTStats {
        self.m_index.stats()
    }

    fn size(&self) -> usize {
        self.m_index.size()
    }

    fn leaves(&self) -> usize {
        self.m_index.leaves()
    }

    fn depth(&self) -> usize {
        self.m_index.max_depth()
    }

    fn split_count(&self) -> usize {
        self.m_index.split_count()
    }

    fn innode_stats(&self) -> impl Iterator<Item = InNodeStats> {
        self.m_index.innode_stats()
    }
}

impl<'a, H> DyftIndex<'a, H>
where
    H: TrajectoryLsh + GetSize,
    H::Hash: GetSize, 
    [H::Hash]: Fill,
    [(); H::Hash::N_DIM]:,
{
    pub fn new(config: &IndexConfig<MartConfig>, max_len: usize) -> DyftIndex<'a, H> {
        let &LshConfig {
            k,
            l,
            resolution,
            seed,
        } = config.lsh_params();
        let mart_config = config.index_params();
        DyftIndex {
            m_hasher: TensoredMultiHash::<H>::init(
                l,
                k,
                resolution,
                max_len,
                &mut StdRng::seed_from_u64(seed),
            ),
            m_index: MartIndex::new(&mart_config, 0, H::Hash::N_DIM.min(u64::N_DIM)),
        }
    }

    pub fn build_with_memory_samples(
        &mut self,
        dataset: &VCodeArray<H::Hash>,
        stats: &mut MasterStats<DyFTStats, IndexConfig<MartConfig>>,
        samples: &[usize],
    ) {
        for &sample in samples {
            self.build(&dataset, sample);
            stats.sample_mem(self.size(), self.index_size());
        }
    }

    pub fn size(&self) -> usize {
        self.m_index.size()
    }

    pub fn trie_query_collect_with_verification<'b, V>(
        &'b self,
        vcodes: &'b VCodeArray<H::Hash>,
        qvcodes: &'b VCodeArray<H::Hash>,
        dataset: &'b [Trajectory],
        queryset: &'b [Trajectory],
        distance: Distance,
    ) -> DyFTPartialVerificationResult<V>
    where
        V: FromIterator<(usize, usize)> + IntoIterator<Item = (usize, usize)>,
    {
        self.m_index.trie_query_partial_verification::<H::Hash, V>(
            &vcodes, &qvcodes, &dataset, &queryset, distance,
        )
    }

    pub fn trie_query_collect<'b, V>(
        &'b self,
        vcodes: &'b VCodeArray<H::Hash>,
        qvcodes: &'b VCodeArray<H::Hash>,
    ) -> V
    where
        V: FromIterator<(usize, usize)>,
    {
        V::from_iter(self.m_index.trie_query(&vcodes, &qvcodes))
    }
}

impl<'a, H> DyftIndex<'a, H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
    [(); H::Hash::N_DIM]:,
{
    pub fn hash_dataset(&'a self, dataset: &'a [Trajectory]) -> VCodeArray<H::Hash> {
        VCodeArray::from_hashes(
            dataset
                .iter()
                .flat_map(|trajectory| self.m_hasher.multi_hash(trajectory)),
            self.m_index.m_bits,
        )
    }

    pub fn hash_queryset(&'a self, dataset: &'a [Trajectory]) -> VCodeArray<H::Hash> {
        VCodeArray::from_hashes(
            dataset
                .iter()
                .flat_map(|trajectory| self.m_hasher.multi_hash_query(trajectory)),
            self.m_index.m_bits,
        )
    }
}

impl<'a, H> MartExporter for DyftIndex<'a, H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
    [(); H::Hash::N_DIM]:,
{
    fn export(self) -> MartTrieExport {
        self.m_index.export()
    }
}

impl<'a, H> GetSize for DyftIndex<'a, H>
where
    H: TrajectoryLsh + GetSize,
    H::Hash: GetSize,
    [(); H::Hash::N_DIM]:,
{
    fn get_heap_size(&self) -> usize {
        self.m_hasher.get_heap_size() + self.m_index.get_heap_size()
    }
}

impl<'a, H> GetIndexSize for DyftIndex<'a, H>
where
    H: TrajectoryLsh + GetSize,
    H::Hash: GetSize,
    [(); H::Hash::N_DIM]:,
{
    fn index_size(&self) -> IndexSize {
        IndexSize {
            stack_size: Self::get_stack_size(),
            heap_size: self.get_heap_size(),
        }
    }
}