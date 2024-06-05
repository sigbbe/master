use anyhow::Result;
use master::config::FreshConfig;
use master::config::FreshPartialVerification;
use master::config::IndexConfig;
use master::fresh::Fresh;
use master::fresh::FreshStats;
use master::io;
use master::lsh::Linear64;
use master::params::IndexCommandWithArgs;
use master::util::MasterStats;

fn main() -> Result<()> {
    sysinfo::set_open_files_limit(0);

    let config = IndexCommandWithArgs::<FreshConfig>::parse_from_args_or_file()?;
    let index_config = config.index_config();
    let index_params = index_config.index_params();
    let datapath = config.data_path();
    let querypath = config.query_path();

    let mut stats = MasterStats::<FreshStats, IndexConfig<FreshConfig>>::new(
        "fresh",
        index_config,
        datapath,
        querypath,
    );

    let start = std::time::SystemTime::now();
    let dataset = io::trajectory_dataset(&datapath)?.take(config.n());
    let data_trajectories = dataset.trajectories();
    let data_ids = dataset.ids();
    let queryset = querypath
        .as_ref()
        .map(|p| io::trajectory_queryset(p))
        .transpose()?;
    stats.data_load_time(start.elapsed()?);

    // instantiate the index
    let dataset_size = dataset.len();
    let max_len = dataset.max_trajectory_length();
    let mut fresh = Fresh::<Linear64>::new(&index_config, dataset_size, max_len);

    // build the index
    let start = std::time::SystemTime::now();
    let fresh = if let Some(samples) = config.memory_samples() {
        fresh.build_with_memory_samples(&data_trajectories, &mut stats, &samples)
    } else {
        fresh.build(&data_trajectories);
        fresh.fix_table()
    };
    stats.index_build_time(start.elapsed()?);
    stats.index_stats(fresh.stats());

    // query the index
    if let Some(queryset) = queryset {
        let query_trajectories = queryset.trajectories();
        let query_ids = queryset.ids();
        let start = std::time::SystemTime::now();
        let results = match index_params.partial_verification() {
            Some(FreshPartialVerification {
                distance,
                verify_fraction,
            }) => {
                let query_stats = stats.index_stats_mut_unchecked();
                let results = fresh.query_with_verification_fraction::<Vec<(usize, usize)>>(
                    data_trajectories,
                    query_trajectories,
                    distance,
                    verify_fraction,
                );
                query_stats.partial_verification_stats(&results);
                results.candidates
            }
            None => fresh.query_collision_collect::<Vec<(usize, usize)>>(&query_trajectories),
        };
        stats.index_query_time(start.elapsed()?);
        let result_pairs: Vec<_> =
            master::util::map_to_trajectory_ids(results, data_ids, query_ids).collect();
        stats.candidates(result_pairs.iter().cloned());
        if let Some(outpath) = config.output_path() {
            io::write_query_results(outpath, result_pairs)?;
        }
        serde_json::to_writer(std::io::stdout(), &stats)?;
    } else {
        serde_json::to_writer(std::io::stdout(), &stats)?;
    }

    Ok(())
}
