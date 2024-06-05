use anyhow::Ok;
use anyhow::Result;
use arrow::datatypes::UInt64Type;
use clap::Parser;
use master::io::trajectory_dataset;
use master::io::write_hashes_parquet;
use master::lsh::MultiTrajectoryLsh;
use master::lsh::TensoredMultiHash;
use master::lsh::Linear64;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct WriteHashesCommand {
    dataset: PathBuf, 
    delta: f64,
    output: PathBuf,
    l: usize,
    k: usize,
    seed: Option<u64>,
    n: Option<usize>,
    max_length: Option<usize>,
}

fn main() -> Result<()> {
    let args = WriteHashesCommand::parse();
    let data = trajectory_dataset(&args.dataset)?;
    let seed = args.seed.unwrap_or(0);
    let n = args.n.unwrap_or(data.len());
    let data = data.take(Some(n));
    let ids = data.ids().into_iter().map(|id| id.value());
    let trajectories = data.trajectories();
	let lsh = TensoredMultiHash::<Linear64>::init(
        args.l,
        args.k,
        args.delta,
        args.max_length.unwrap_or(data.max_trajectory_length()),
        &mut StdRng::seed_from_u64(seed),
    );
    let start = std::time::SystemTime::now();
    let hashes = trajectories.into_iter().map(|t| lsh.multi_hash(&t));
    write_hashes_parquet::<UInt64Type>(
        args.output,
        hashes.into_iter(),
        ids,
    )?;
    println!("Wrote hashes in {:?}", start.elapsed()?);
    Ok(())
}