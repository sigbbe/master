#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::measurement::Measurement;
use criterion::AxisScale;
use criterion::Bencher;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::PlotConfiguration;
use criterion::Throughput;
use master::benchmarks::IndexBenchmarkBuildSetup;
use master::benchmarks::IndexBenchmarkQuerySetup;
use master::benchmarks::BACKWARD_DATASET;
use master::benchmarks::BACKWARD_PERCENTILES_GRID;
use master::benchmarks::BACKWARD_QUERYSET;
use master::benchmarks::MAX_BENCHMARK_INDEX_SIZE;
use master::benchmarks::N_SAMPLES;
use master::benchmarks::PORTO_DATASET;
use master::benchmarks::PORTO_DATASET_FULL;
use master::benchmarks::PORTO_PERCENTILES_GRID;
use master::benchmarks::PORTO_QUERYSET;
use master::config::IndexConfig;
use master::config::MartConfig;
use master::dyft::DyFT;
use master::dyft::DyftIndex;
use master::dyft::VCodeArray;
use master::dyft::VCodeTools;
use master::lsh::Linear64;
use master::lsh::TrajectoryLsh;
use master::point::Distance;
use master::trajectory::TrajectoryDataset;
use rand::Fill;
use std::path::Path;
use std::time::Duration;
use get_size::GetSize;

mod build {
    use master::benchmarks::BACKWARD_DATASET_FULL;
    use super::*;

    fn dyft_build_procedure<'a, M>(
        b: &mut Bencher<'a, M>,
        instance: &IndexBenchmarkBuildSetup<MartConfig>,
        n: usize,
    ) where
        M: Measurement,
    {
        b.iter(|| {
            let mut index = DyftIndex::<Linear64>::new(
                black_box(&instance.config),
                black_box(instance.max_len),
            );
            let vcodes = index.hash_dataset(black_box(
                &instance.dataset.trajectories()[..MAX_BENCHMARK_INDEX_SIZE.min(n)],
            ));
            index.build(
                black_box(&vcodes),
                black_box(
                    MAX_BENCHMARK_INDEX_SIZE
                        .min(vcodes.size())
                        .try_into()
                        .unwrap(),
                ),
            );
        });
    }

    macro_rules! build_bench {
        ($fun:ident, $name:expr, $config:expr, $dataset:expr) => {
            pub fn $fun(c: &mut Criterion) {
                let instance = IndexBenchmarkBuildSetup::<MartConfig>::load_build_benchmark_inputs(
                    $config, $dataset,
                );
                let plot_config =
                    PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
                let n = instance.dataset.len();
                let mut group = c.benchmark_group($name);
                group.plot_config(plot_config);
                group.throughput(Throughput::Elements(instance.dataset.len() as u64));
                for &i in N_SAMPLES.iter() {
                    if i < n {
                        group.bench_function(BenchmarkId::from_parameter(i), |b| {
                            dyft_build_procedure(b, &instance, i)
                        });
                    } else {
                        group.bench_function(BenchmarkId::from_parameter(n), |b| {
                            dyft_build_procedure(b, &instance, n)
                        });
                        break;
                    }
                }
                group.finish();
            }
        };
    }

    build_bench!(
        porto_build_1,
        "dyft-build-benchmark-DBP-1",
        IndexConfig::<MartConfig>::default().resolution(PORTO_PERCENTILES_GRID[2]),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_2,
        "dyft-build-benchmark-DBP-2",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .with_in_weight(0.1),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_3,
        "dyft-build-benchmark-DBP-3",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .with_in_weight(10.0),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_4,
        "dyft-build-benchmark-DBP-4",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .with_errors(4),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_5,
        "dyft-build-benchmark-DBP-5",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .with_errors(16),
        PORTO_DATASET_FULL
    );

    build_bench!(
        backward_build_1,
        "dyft-build-benchmark-DBB-1",
        IndexConfig::<MartConfig>::default().resolution(BACKWARD_PERCENTILES_GRID[2]),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_2,
        "dyft-build-benchmark-DBB-2",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .with_in_weight(0.1),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_3,
        "dyft-build-benchmark-DBB-3",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .with_in_weight(10.0),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_4,
        "dyft-build-benchmark-DBB-4",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .with_errors(4),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_5,
        "dyft-build-benchmark-DBB-5",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .with_errors(16),
        BACKWARD_DATASET_FULL
    );
}

mod query {
    use super::*;

    struct DyFTQueryInstance<'a, H>
    where
        H: TrajectoryLsh,
        [H::Hash]: Fill,
        [(); H::Hash::N_DIM]:,
    {
        index: DyftIndex<'a, H>,
        vcodes: VCodeArray<H::Hash>,
        #[allow(dead_code)]
        dataset: TrajectoryDataset<'a>,
        queryset: TrajectoryDataset<'a>,
        #[allow(dead_code)]
        distance: Option<Distance>,
    }

    fn prepare_dyft_query_instance<'a, H>(
        config: IndexConfig<MartConfig>,
        dataset: impl AsRef<Path>,
        queryset: impl AsRef<Path>,
    ) -> DyFTQueryInstance<'a, H>
    where
        H: TrajectoryLsh + GetSize,
        H::Hash: GetSize,
        [H::Hash]: Fill,
        [(); H::Hash::N_DIM]:,
    {
        let IndexBenchmarkQuerySetup {
            build:
                IndexBenchmarkBuildSetup {
                    config,
                    dataset,
                    max_len: max_len_data,
                },
            queryset,
            max_len: max_len_query,
        } = IndexBenchmarkQuerySetup::<MartConfig>::load_query_benchmark_inputs(
            config, dataset, queryset,
        );
        let distance = config.index_params().distance;
        let max_len = max_len_data.max(max_len_query);
        let mut index = DyftIndex::<H>::new(&config, max_len);
        let vcodes = index.hash_dataset(dataset.trajectories());
        index.build(
            &vcodes,
            MAX_BENCHMARK_INDEX_SIZE
                .min(vcodes.size())
                .try_into()
                .unwrap(),
        );

        DyFTQueryInstance {
            index,
            vcodes,
            dataset,
            queryset,
            distance,
        }
    }

    fn dyft_query_no_verification<'a, M, H>(
        c: &'a mut Criterion<M>,
        instance: &DyFTQueryInstance<'a, H>,
        id: &str,
    ) -> &'a mut Criterion<M>
    where
        M: Measurement + 'static,
        H: TrajectoryLsh + GetSize,
        H::Hash: GetSize,
        [(); H::Hash::N_DIM]:,
        [H::Hash]: Fill,
    {
        c.bench_function(id, |b: &mut Bencher<'_, M>| {
            b.iter(|| {
                let qvcodes = instance
                    .index
                    .hash_queryset(&instance.queryset.trajectories());
                let _ = instance.index.trie_query_collect::<Vec<(usize, usize)>>(
                    black_box(&instance.vcodes),
                    black_box(&qvcodes),
                );
            })
        })
    }

    macro_rules! query_bench {
        ($fun:ident, $bench_fn:ident, $name:expr, $config:expr, $dataset:expr, $queryset:expr) => {
            pub fn $fun(c: &mut Criterion) {
                let instance =
                    prepare_dyft_query_instance::<Linear64>($config, $dataset, $queryset);
                $bench_fn(c, &instance, $name);
            }
        };
    }

    query_bench!(
        porto_query_1,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQP-1",
        IndexConfig::<MartConfig>::default().resolution(PORTO_PERCENTILES_GRID[0]),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench!(
        porto_query_2,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQP-2",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .with_in_weight(0.1),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench!(
        porto_query_3,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQP-3",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .with_in_weight(10.0),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench!(
        porto_query_4,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQP-4",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .with_errors(4),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench!(
        porto_query_5,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQP-5",
        IndexConfig::<MartConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .with_errors(16),
        PORTO_DATASET,
        PORTO_QUERYSET
    );

    query_bench!(
        backward_query_1,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQB-1",
        IndexConfig::<MartConfig>::default().resolution(BACKWARD_PERCENTILES_GRID[0]),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench!(
        backward_query_2,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQB-2",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .with_in_weight(0.1),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );

    query_bench!(
        backward_query_3,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQB-3",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .with_in_weight(10.0),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench!(
        backward_query_4,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQB-4",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .with_errors(4),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench!(
        backward_query_5,
        dyft_query_no_verification,
        "dyft-query-benchmark-DQB-5",
        IndexConfig::<MartConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .with_errors(16),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
}

criterion_group!(
    name = dyft_build_benches;
    config = Criterion::default()
        .sample_size(20)
        .with_plots()
        .warm_up_time(Duration::from_secs(5))
        .save_baseline(String::from("dyft-build-baseline"));
    targets =
        build::porto_build_1,
        build::porto_build_2,
        build::porto_build_3,
        build::porto_build_4,
        build::porto_build_5,
        build::backward_build_1,
        build::backward_build_2,
        build::backward_build_3,
        build::backward_build_4,
        build::backward_build_5
);

criterion_group!(
    name = dyft_query_benches;
    config = Criterion::default()
        .sample_size(20)
        .with_plots()
        .warm_up_time(Duration::from_secs(5))
        .save_baseline(String::from("dyft-query-baseline"));
    targets =
        query::porto_query_1,
        query::porto_query_2,
        query::porto_query_3,
        query::porto_query_4,
        query::porto_query_5,
        query::backward_query_1,
        query::backward_query_2,
        query::backward_query_3,
        query::backward_query_4,
        query::backward_query_5
);

criterion_main!(dyft_build_benches, dyft_query_benches);
