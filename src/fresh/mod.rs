use self::multi::MultiTable;
use self::table::LSHTable;
use crate::config::FreshConfig;
use crate::config::IndexConfig;
use crate::config::LshConfig;
use crate::frechet::FrechetDistanceFilter;
use crate::lsh::CurveToIdx;
use crate::lsh::MultiTrajectoryLsh;
use crate::lsh::TensoredMultiHash;
use crate::lsh::TrajectoryLsh;
use crate::point::Distance;
use crate::trajectory::Trajectory;
use crate::util::IndexSize;
use crate::util::MasterStats;
use get_size::GetSize;
use itertools::FoldWhile;
use itertools::Itertools;
use num_traits::ToPrimitive;
use rand::rngs::StdRng;
use rand::Fill;
use rand::SeedableRng;
use serde::Serialize;

mod multi;
mod table;

pub struct Fresh<H, const B: bool = false>
where
    H: TrajectoryLsh,
{
    m_hasher: TensoredMultiHash<H>,
    m_lsh_table: MultiTable<H, B>,
    m_tau: Option<f64>,
}

impl<H> Fresh<H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
{
    pub fn new(config: &IndexConfig<FreshConfig>, num_trajectories: usize, max_len: usize) -> Self {
        let &LshConfig {
            k,
            l,
            resolution,
            seed,
        } = config.lsh_params();
        let &FreshConfig {
            verify_fraction, ..
        } = config.index_params();
        let tables = Vec::from_iter((0..l).map(|_| LSHTable::new(num_trajectories, max_len)));
        let rng: &mut StdRng = &mut SeedableRng::seed_from_u64(seed);
        Fresh::<H, false> {
            m_hasher: TensoredMultiHash::<H>::init(l, k, resolution, max_len, rng),
            m_lsh_table: MultiTable::<H>::new(tables, max_len),
            m_tau: verify_fraction,
        }
    }

    pub fn build(&mut self, dataset: &[Trajectory]) {
        self.m_lsh_table.put(&self.m_hasher, &dataset);
    }

    pub fn fix_table(self) -> Fresh<H, true> {
        Fresh::<H, true> {
            m_hasher: self.m_hasher,
            m_lsh_table: self.m_lsh_table.fix_table(),
            m_tau: self.m_tau,
        }
    }

    pub fn build_with_memory_samples<'a>(
        self,
        dataset: &[Trajectory],
        stats: &mut MasterStats<FreshStats, IndexConfig<FreshConfig>>,
        samples: &[usize],
    ) -> Fresh<H, true> {
        let mut i = 0;
        let mut fresh = self;
        samples.iter().for_each(|&j| {
            fresh.m_lsh_table.put(&fresh.m_hasher, &dataset[i..j]);
            stats.sample_mem(j, fresh.sample_memory());
            i = j;
        });
        let fresh = Fresh::<H, true> {
            m_hasher: fresh.m_hasher,
            m_lsh_table: fresh.m_lsh_table.fix_table(),
            m_tau: fresh.m_tau,
        };
        stats.sample_mem(fresh.m_lsh_table.size(), fresh.sample_memory());
        fresh 
    }
}
impl<'a, H, const B: bool> Fresh<H, B>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
{
    pub fn sample_memory(&self) -> IndexSize {
        IndexSize {
            stack_size: self.get_size(),
            heap_size: self.get_heap_size(),
        }
    }
}

impl<'a, H> Fresh<H, true>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
{
    pub fn query_collision_collect<V>(&'a self, queryset: &'a [Trajectory]) -> V
    where
        V: FromIterator<(usize, usize)> + 'a,
    {
        V::from_iter(self.m_lsh_table.query_iter(&self.m_hasher, queryset))
    }

    pub fn query_with_verification(
        &'a self,
        dataset: &'a [Trajectory],
        queryset: &'a [Trajectory],
        distance: Distance,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        self.m_lsh_table
            .query_iter(&self.m_hasher, queryset)
            .frechet_distance_filter(dataset, queryset, distance)
    }

    pub fn query_with_verification_fraction<V>(
        &'a self,
        dataset: &'a [Trajectory],
        queryset: &'a [Trajectory],
        distance: f64,
        verify_fraction: f64,
    ) -> FreshPartialVerificationResult<V>
    where
        V: FromIterator<(usize, usize)>,
    {
        let num_buckets = self.m_hasher.hash_length().to_f64().unwrap();
        let scored_candidates: Vec<(Distance, CurveToIdx<usize>)> = self
            .m_lsh_table
            .query_count_scores(&self.m_hasher, queryset)
            .collect();

        let upper_score = self.estimate_thresholds(
            scored_candidates.len() as f64,
            num_buckets,
            verify_fraction,
            scored_candidates.iter().map(|(score, _pair)| score),
        );

        FreshPartialVerificationResult::<V>::from_scored_results(
            scored_candidates,
            dataset,
            queryset,
            distance,
            num_buckets,
            upper_score,
        )
    }

    /// Estimate the thresholds for the verification step
    /// returns the upper score threshold for the trajectories
    /// to be verified
    fn estimate_thresholds<'b, I>(
        &self,
        num_candidates: f64,
        num_buckets: f64,
        verify_fraction: f64,
        scored_pairs: I,
    ) -> f64
    where
        I: IntoIterator<Item = &'b f64>,
    {
        let l = self.m_hasher.hash_length();
        let mut hist = vec![0; l];
        scored_pairs
            .into_iter()
            .filter_map(|&score| score.round().to_usize().map(|b| b.min(l)))
            .for_each(|bucket| hist[bucket - 1] += 1);

        let count_threshold = (num_candidates * verify_fraction).round() as u32;

        let score = (0..)
            .fold_while((0f64, 0u32), |(i, count), j| {
                if count < count_threshold {
                    FoldWhile::Continue((i + 1.0, count + hist[j]))
                } else {
                    FoldWhile::Done((i, count))
                }
            })
            .into_inner()
            .0;

        score / num_buckets
    }

    pub fn stats(&self) -> FreshStats {
        FreshStats {
            size: self.m_lsh_table.size(),
            k: self.m_hasher.concatenations(),
            l: self.m_hasher.hash_length(),
            distinct: self.m_lsh_table.distinct_hashes(),
            skipped_verification: 0,
            verified_results: 0,
            verified_filtered: 0,
        }
    }
}

pub struct FreshPartialVerificationResult<V> {
    pub full_verify_filtered: usize,
    pub full_verify_unfiltered: usize,
    pub skipped_verification: usize,
    pub candidates: V,
}

impl<V> FreshPartialVerificationResult<V>
where
    V: FromIterator<(usize, usize)>,
{
    pub fn from_scored_results(
        scored_results: impl IntoIterator<Item = (Distance, (usize, usize))>,
        dataset: &[Trajectory],
        queryset: &[Trajectory],
        distance: Distance,
        num_buckets: Distance,
        upper_score: Distance,
    ) -> Self {
        let mut full_verify_filtered = 0;
        let mut skipped_verification = 0;
        let mut full_verify_unfiltered = 0;
        let candidates = scored_results
            .into_iter()
            .filter_map(move |(score, pair)| {
                if score > 0.0 {
                    Some((score / num_buckets, pair))
                } else {
                    None
                }
            })
            .filter(|&(score, (qid, tid))| match score > upper_score {
                true => {
                    skipped_verification += 1;
                    true
                }
                false => {
                    if queryset[qid].frechet_decider(&dataset[tid], distance) {
                        full_verify_unfiltered += 1;
                        true
                    } else {
                        full_verify_filtered += 1;
                        false
                    }
                }
            })
            .map(|(_score, (qid, tid))| (qid, tid))
            .collect();

        FreshPartialVerificationResult {
            candidates,
            full_verify_filtered,
            skipped_verification,
            full_verify_unfiltered,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FreshStats {
    pub size: usize,
    pub k: usize,
    pub l: usize,
    pub distinct: Vec<usize>,
    pub skipped_verification: usize,
    pub verified_results: usize,
    pub verified_filtered: usize,
}

impl FreshStats {
    pub fn partial_verification_stats<V>(&mut self, results: &FreshPartialVerificationResult<V>) {
        self.skipped_verification = results.skipped_verification;
        self.verified_results = results.full_verify_unfiltered;
        self.verified_filtered = results.full_verify_filtered;
    }
}

impl<H, const B: bool> GetSize for Fresh<H, B>
where
    H: TrajectoryLsh + GetSize,
    H::Hash: GetSize,
{
    fn get_heap_size(&self) -> usize {
        self.m_hasher.get_heap_size()
            + self.m_lsh_table.get_heap_size()
            + self.m_tau.get_heap_size()
    }
}
