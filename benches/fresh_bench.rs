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
use master::benchmarks::BACKWARD_DATASET_FULL;
use master::benchmarks::BACKWARD_PERCENTILES_GRID;
use master::benchmarks::BACKWARD_QUERYSET;
use master::benchmarks::MAX_BENCHMARK_INDEX_SIZE;
use master::benchmarks::N_SAMPLES;
use master::benchmarks::PORTO_DATASET;
use master::benchmarks::PORTO_PERCENTILES_GRID;
use master::benchmarks::PORTO_QUERYSET;
use master::config::FreshConfig;
use master::config::IndexConfig;
use master::dyft::VCodeTools;
use master::fresh::Fresh;
use master::lsh::Linear64;
use master::lsh::TrajectoryLsh;
use master::point::Distance;
use master::trajectory::TrajectoryDataset;
use rand::Fill;
use std::path::Path;
use std::time::Duration;

mod build {
    use master::benchmarks::PORTO_DATASET_FULL;

    use super::*;

    fn fresh_build_procedure<'a, M>(
        b: &mut Bencher<'a, M>,
        instance: &IndexBenchmarkBuildSetup<FreshConfig>,
        n: usize,
    ) where
        M: Measurement,
    {
        b.iter(|| {
            let _ = Fresh::<Linear64>::new(
                black_box(&instance.config),
                black_box(n),
                black_box(instance.max_len),
            )
            .build(black_box(&instance.dataset.trajectories()[..n]));
        });
    }

    macro_rules! build_bench {
        ($fun:ident, $name:expr, $config:expr, $dataset:expr) => {
            pub fn $fun(c: &mut Criterion) {
                let instance = IndexBenchmarkBuildSetup::<FreshConfig>::load_build_benchmark_inputs(
                    $config, $dataset,
                );
                let plot_config =
                    PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
                let n = instance.dataset.len();
                let mut group = c.benchmark_group($name);
                group.plot_config(plot_config);
                for &i in N_SAMPLES.iter() {
                    if i < n {
                        group.throughput(Throughput::Elements(i as u64));
                        group.bench_function(BenchmarkId::from_parameter(i), |b| {
                            fresh_build_procedure(b, &instance, i)
                        });
                    } else {
                        group.throughput(Throughput::Elements(n as u64));
                        group.bench_function(BenchmarkId::from_parameter(n), |b| {
                            fresh_build_procedure(b, &instance, n)
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
        "fresh-build-benchmark-FBP-1",
        IndexConfig::<FreshConfig>::default().resolution(PORTO_PERCENTILES_GRID[2]),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_2,
        "fresh-build-benchmark-FBP-2",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .k(4),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_3,
        "fresh-build-benchmark-FBP-3",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .k(8),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_4,
        "fresh-build-benchmark-FBP-4",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .l(64),
        PORTO_DATASET_FULL
    );
    build_bench!(
        porto_build_5,
        "fresh-build-benchmark-FBP-5",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[2])
            .k(4)
            .l(128),
        PORTO_DATASET_FULL
    );

    build_bench!(
        backward_build_1,
        "fresh-build-benchmark-FBB-1",
        IndexConfig::<FreshConfig>::default().resolution(BACKWARD_PERCENTILES_GRID[2]),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_2,
        "fresh-build-benchmark-FBB-2",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .k(4),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_3,
        "fresh-build-benchmark-FBB-3",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .k(8),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_4,
        "fresh-build-benchmark-FBB-4",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .l(64),
        BACKWARD_DATASET_FULL
    );
    build_bench!(
        backward_build_5,
        "fresh-build-benchmark-FBB-5",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[2])
            .k(4)
            .l(128),
        BACKWARD_DATASET_FULL
    );
}

mod query {
    use super::*;

    struct FreshQueryInstance<'a, H>
    where
        H: TrajectoryLsh,
        [H::Hash]: Fill,
        [(); H::Hash::N_DIM]:,
    {
        index: Fresh<H, true>,
        dataset: TrajectoryDataset<'a>,
        queryset: TrajectoryDataset<'a>,
        verify_fraction_distance: Option<(Distance, Distance)>,
    }

    fn prepare_fresh_query_instance<'a, H>(
        config: IndexConfig<FreshConfig>,
        dataset: impl AsRef<Path>,
        queryset: impl AsRef<Path>,
    ) -> FreshQueryInstance<'a, H>
    where
        H: TrajectoryLsh,
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
        } = IndexBenchmarkQuerySetup::<FreshConfig>::load_query_benchmark_inputs(
            config, dataset, queryset,
        );
        let distance = config.index_params().distance;
        let max_len = max_len_data.max(max_len_query);
        let mut index = Fresh::new(&config, max_len, dataset.len());
        index.build(&dataset.trajectories()[..MAX_BENCHMARK_INDEX_SIZE.min(dataset.len())]);
        let index = index.fix_table();
        let verify_fraction_distance = match config.index_params().verify_fraction {
            Some(verify_fraction) => match distance {
                Some(distance) => Some((verify_fraction, distance)),
                None => None,
            },
            None => None,
        };
        FreshQueryInstance {
            index,
            dataset,
            queryset,
            verify_fraction_distance,
        }
    }

    fn fresh_query_with_verification<'a, M, H>(
        b: &mut Bencher<'a, M>,
        instance: &FreshQueryInstance<'a, H>,
    ) where
        M: Measurement,
        H: TrajectoryLsh,
        [(); H::Hash::N_DIM]:,
        [H::Hash]: Fill,
    {
        let (verify_fraction, distance) = instance.verify_fraction_distance.unwrap();
        b.iter(|| {
            instance
                .index
                .query_with_verification_fraction::<Vec<(usize, usize)>>(
                    black_box(instance.dataset.trajectories()),
                    black_box(instance.queryset.trajectories()),
                    black_box(distance),
                    black_box(verify_fraction),
                )
        });
    }

    fn fresh_query_no_verification<'a, M, H>(
        b: &mut Bencher<'a, M>,
        instance: &FreshQueryInstance<'a, H>,
    ) where
        M: Measurement,
        H: TrajectoryLsh,
        [(); H::Hash::N_DIM]:,
        [H::Hash]: Fill,
    {
        b.iter(|| {
            instance
                .index
                .query_collision_collect::<Vec<(usize, usize)>>(black_box(
                    &instance.queryset.trajectories(),
                ))
        })
    }

    macro_rules! query_bench_no_verify {
        ($fun:ident, $name:expr, $config:expr, $dataset:expr, $queryset:expr) => {
            pub fn $fun(c: &mut Criterion) {
                let instance =
                    prepare_fresh_query_instance::<Linear64>($config, $dataset, $queryset);
                let n = instance.dataset.len();
                let plot_config =
                    PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
                let mut group = c.benchmark_group($name);
                group.plot_config(plot_config);
                for &i in N_SAMPLES.iter() {
                    if i < n {
                        group.bench_function(BenchmarkId::from_parameter(i), |b| {
                            fresh_query_no_verification(b, &instance)
                        });
                    } else {
                        group.bench_function(BenchmarkId::from_parameter(n), |b| {
                            fresh_query_no_verification(b, &instance)
                        });
                        break;
                    }
                }
                group.finish();
            }
        };
    }

    macro_rules! query_bench_verify {
        ($fun:ident, $name:expr, $config:expr, $dataset:expr, $queryset:expr) => {
            pub fn $fun(c: &mut Criterion) {
                let instance =
                    prepare_fresh_query_instance::<Linear64>($config, $dataset, $queryset);
                let n = instance.dataset.len();
                let plot_config =
                    PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
                let mut group = c.benchmark_group($name);
                group.plot_config(plot_config);
                for &i in N_SAMPLES.iter() {
                    if i < n {
                        group.bench_function(BenchmarkId::from_parameter(i), |b| {
                            fresh_query_with_verification(b, &instance)
                        });
                    } else {
                        group.bench_function(BenchmarkId::from_parameter(n), |b| {
                            fresh_query_with_verification(b, &instance)
                        });
                        break;
                    }
                }
                group.finish();
            }
        };
    }

    query_bench_no_verify!(
        porto_query_1,
        "fresh-query-benchmark-FQP-1",
        IndexConfig::<FreshConfig>::default().resolution(PORTO_PERCENTILES_GRID[0]),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench_no_verify!(
        porto_query_2,
        "fresh-query-benchmark-FQP-2",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .k(4),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench_no_verify!(
        porto_query_3,
        "fresh-query-benchmark-FQP-3",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .k(8),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench_no_verify!(
        porto_query_4,
        "fresh-query-benchmark-FQP-4",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .l(64),
        PORTO_DATASET,
        PORTO_QUERYSET
    );
    query_bench_verify!(
        porto_query_5,
        "fresh-query-benchmark-FQP-5",
        IndexConfig::<FreshConfig>::default()
            .resolution(PORTO_PERCENTILES_GRID[0])
            .k(4)
            .l(128)
            .verify_fraction(1.0 / 4.0)
            .distance(PORTO_PERCENTILES_GRID[0] / 8.0),
        PORTO_DATASET,
        PORTO_QUERYSET
    );

    query_bench_no_verify!(
        backward_query_1,
        "fresh-query-benchmark-FQB-1",
        IndexConfig::<FreshConfig>::default().resolution(BACKWARD_PERCENTILES_GRID[0]),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench_no_verify!(
        backward_query_2,
        "fresh-query-benchmark-FQB-2",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .k(4),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench_no_verify!(
        backward_query_3,
        "fresh-query-benchmark-FQB-3",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .k(8),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench_no_verify!(
        backward_query_4,
        "fresh-query-benchmark-FQB-4",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .l(64),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
    query_bench_verify!(
        backward_query_5,
        "fresh-query-benchmark-FQB-5",
        IndexConfig::<FreshConfig>::default()
            .resolution(BACKWARD_PERCENTILES_GRID[0])
            .k(4)
            .l(128)
            .verify_fraction(1.0 / 4.0)
            .distance(BACKWARD_PERCENTILES_GRID[0] / 8.0),
        BACKWARD_DATASET,
        BACKWARD_QUERYSET
    );
}

criterion_group!(
    name = fresh_build_benches;
    config = Criterion::default()
        .sample_size(20)
        .with_plots()
        .warm_up_time(Duration::from_secs(5));
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
    name = fresh_query_benches;
    config = Criterion::default()
        .sample_size(20)
        .with_plots()
        .warm_up_time(Duration::from_secs(5));
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
criterion_main!(fresh_build_benches, fresh_query_benches);