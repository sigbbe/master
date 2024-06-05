use crate::config::IndexConfig;
use crate::io::trajectory_dataset;
use crate::io::trajectory_queryset;
use crate::trajectory::TrajectoryDataset;
use clap::Args;
use serde::Deserialize;
use std::path::Path;

pub const MAX_BENCHMARK_INDEX_SIZE: usize = 100_000;
pub const N_SAMPLES: [usize; 3] = [1_000, 10_000, 100_000];

// 1st, 5th and 10th percentiles of the Porto and Backward datasets
pub const PORTO_PERCENTILES_GRID: [f64; 3] = [8.0 * 0.011241832018847207, 8.0 * 0.018916974199115043, 8.0 * 0.02379386376568472];
pub const BACKWARD_PERCENTILES_GRID: [f64; 3] = [8.0 * 8.97477487182388, 8.0 * 14.210116114937271, 8.0 * 17.914667175250568];

pub const PORTO_DATASET_FULL: &str = "porto.parquet";
pub const PORTO_DATASET: &str = "porto-data.parquet";
pub const PORTO_QUERYSET: &str = "porto-query.parquet";

pub const BACKWARD_DATASET_FULL: &str = "backward.parquet";
pub const BACKWARD_DATASET: &str = "backward-data.parquet";
pub const BACKWARD_QUERYSET: &str = "backward-query.parquet";


pub struct IndexBenchmarkBuildSetup<'a, C>
where
    C: Args,
{
    pub config: IndexConfig<C>,
    pub dataset: TrajectoryDataset<'a>,
    pub max_len: usize,
}

pub struct IndexBenchmarkQuerySetup<'a, C>
where
    C: Args,
{
    pub build: IndexBenchmarkBuildSetup<'a, C>,
    pub queryset: TrajectoryDataset<'a>,
    pub max_len: usize,
}

impl<'a, C> IndexBenchmarkBuildSetup<'a, C>
where
    C: Args + for <'de> Deserialize<'de>,
{
    pub fn load_build_benchmark_inputs(
        config: IndexConfig<C>,
        dataset: impl AsRef<Path>,
    ) -> Self {
        let dataset = trajectory_dataset(dataset)
            .expect("[IndexBenchmarkBuildSetup]: failed to load dataset");
        let max_len = dataset.max_trajectory_length();
        IndexBenchmarkBuildSetup::<'a, C> {
            config,
            dataset,
            max_len,
        }
    }
}


impl<'a, C> IndexBenchmarkQuerySetup<'a, C>
where
    C: Args + for <'de> Deserialize<'de>,
{
    pub fn load_query_benchmark_inputs(
        config: IndexConfig<C>,
        dataset: impl AsRef<Path>,
        queryset: impl AsRef<Path>,
    ) -> Self {
        let build = IndexBenchmarkBuildSetup::load_build_benchmark_inputs(config, dataset);
        let queryset = trajectory_queryset(queryset)
            .expect("[IndexBenchmarkQuerySetup]: failed to load queryset");
        let max_len = queryset.max_trajectory_length();
        IndexBenchmarkQuerySetup::<'a, C> {
            build,
            queryset,
            max_len,
        }
    }
}