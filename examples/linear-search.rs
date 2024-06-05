use anyhow::Result;
use master::config::*;
use master::dyft::DyftIndex;
use master::lsh::Linear128;
use master::params::IndexCommandWithArgs;
use master::io::*;

fn main() -> Result<()> {
    let config = IndexCommandWithArgs::<MartConfig>::parse_from_args_or_file()?;
    let datapath = config.data_path();
    let querypath = config.query_path();
	let data = trajectory_dataset(datapath)?;
	if let Some(query) = querypath.map(|p| trajectory_queryset(p)).transpose()? {
		let max_len = data.max_trajectory_length().max(query.max_trajectory_length());
		let dyft = DyftIndex::<Linear128>::new(&config.index_config(), max_len);
		let vcodes = dyft.hash_dataset(data.trajectories());
		let qvcodes = dyft.hash_queryset(query.trajectories());

		let res = vcodes.linear_search(&qvcodes, 10).collect::<Vec<_>>();
		println!("{:?}", res.len());
	}
    Ok(())
}
