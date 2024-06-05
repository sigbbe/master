use anyhow::Result;
use master::config::*;
use master::dyft::DyFT;
use master::dyft::DyFTStatistics;
use master::dyft::DyFTStats;
use master::dyft::DyftIndex;
use master::io;
use master::lsh::*;
use master::params::IndexCommandWithArgs;
use master::util::MasterStats;
use serde_json::to_writer;

fn main() -> Result<()> {
    sysinfo::set_open_files_limit(0);

    let config = IndexCommandWithArgs::<MartConfig>::parse_from_args_or_file()?;
    let index_config = config.index_config();
    let datapath = config.data_path();
    let querypath = config.query_path();
    let mut stats = MasterStats::<DyFTStats, IndexConfig<MartConfig>>::new(
        "dyft",
        index_config,
        datapath,
        querypath,
    );

    let start = std::time::SystemTime::now();
    let dataset = io::trajectory_dataset(datapath)?.take(config.n());
    let data_trajectories = dataset.trajectories();
    let data_ids = dataset.ids();
    let queryset = querypath
        .as_ref()
        .map(|p| io::trajectory_queryset(p))
        .transpose()?;
    stats.data_load_time(start.elapsed()?);

    // instantiate the index
    let mut dyft =
        DyftIndex::<Linear64>::new(&config.index_config(), dataset.max_trajectory_length());

    // build the index
    let vcodes = dyft.hash_dataset(dataset.trajectories());
    let start = std::time::SystemTime::now();
        if let Some(samples) = config.memory_samples() {
        dyft.build_with_memory_samples(&vcodes, &mut stats, &samples);
    } else {
        dyft.build(&vcodes, vcodes.size());
    }
    stats.index_build_time(start.elapsed()?);
    stats.index_stats(dyft.stats());

    // query the index
    if let Some(queryset) = queryset {
        let query_trajectories = queryset.trajectories();
        let query_ids = queryset.ids();
        stats.index_query_size(queryset.len());
        let qvcodes = dyft.hash_queryset(query_trajectories);
        let start = std::time::SystemTime::now();
        let results: Vec<(usize, usize)> = match config.index_config().index_params().distance {
            Some(distance) => {
                let results = dyft.trie_query_collect_with_verification(&vcodes, &qvcodes, data_trajectories, query_trajectories, distance);
                let index_stats = stats.index_stats_mut_unchecked();
                index_stats.partial_verification_count(results.partial_verification_count());
                index_stats.full_verification_count(results.full_verification_count());
                index_stats.filtered_verification_count(results.filtered_verification_count());
                results.results()
            }, 
            None => {
                dyft.trie_query_collect(&vcodes, &qvcodes)
            }
        };
        stats.index_query_time(start.elapsed()?);
        let result_pairs: Vec<_> =
            master::util::map_to_trajectory_ids(results, data_ids, query_ids).collect();
        stats.candidates(result_pairs.iter().cloned());
        if let Some(outpath) = config.output_path() {
            io::write_query_results(outpath, result_pairs)?;
        }
        to_writer(std::io::stdout(), &stats)?;
    } else {
        to_writer(std::io::stdout(), &stats)?;
    }

    Ok(())
}
