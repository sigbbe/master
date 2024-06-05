use anyhow::Result;
use arrow::datatypes::UInt64Type;
use clap::Parser;
use clap::ValueEnum;
use master::dyft::VCodeTools;
use master::io::write_hashes_parquet;
use master::lsh::ConstantFactorLsh;
use master::lsh::LinearFactorLsh;
use master::lsh::MultiTrajectoryLsh;
use master::lsh::TensoredMultiHash;
use master::trajectory::Trajectory;
use rand::rngs::StdRng;
use rand::Fill;
use rand::SeedableRng;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
enum HashType {
    LinearFactorLsh,
    #[default]
    ConstantFactorLsh,
}

#[derive(Debug, Parser, Clone)]
struct HashConfig {
    #[arg(short, long, help = "Type of hash function to use")]
    hash_type: HashType,
    
	#[arg(
        short,
        long,
        help = "Path to the dataset (required) in parquet format. See the README for more information."
    )]
    dataset: PathBuf,

    #[arg(short, long, help = "Resolution of the hash function")]
    resolution: f64,

    #[arg(short, long, help = "Length of the hash")]
    length: usize,

    #[arg(short, long, help = "Number of hash concatentations")]
    concatentations: usize,

    #[arg(
        short,
        long,
        help = "Seed for the random number generator",
        default_value_t = 0
    )]
    seed: u64,

	#[arg(short, long, help = "Path to the output file (required)")]
	output: PathBuf,
}

fn hash<T>(
    hash_type: HashType,
    dataset: &[Trajectory],
    length: usize,
    concatentations: usize,
    resolution: f64,
    seed: u64,
) -> Vec<Vec<T>>
where
    T: VCodeTools,
    [T]: Fill,
{
    let max_len = dataset.iter().map(|t| t.len()).max().unwrap_or(0);
    let rng = &mut StdRng::seed_from_u64(seed);
    match hash_type {
        HashType::LinearFactorLsh => {
            let lsh = TensoredMultiHash::<LinearFactorLsh<T>>::init(length, concatentations, resolution, max_len, rng);
            dataset.into_iter().map(|t| lsh.multi_hash(t).collect()).collect()
        }
        HashType::ConstantFactorLsh => {
            let lsh =
                TensoredMultiHash::<ConstantFactorLsh<T>>::init(length, concatentations, resolution, max_len, rng);
            dataset.into_iter().map(|t| lsh.multi_hash(t).collect()).collect()
        }
    }
}

fn main() -> Result<()> {
    let HashConfig {
        hash_type,
        dataset,
        resolution,
        length, 
        concatentations,
        seed,
		output,
    } = HashConfig::parse();
    let dataset = master::io::trajectory_dataset(&dataset)?;
    let hashes = hash::<u64>(hash_type, dataset.trajectories(), length, concatentations, resolution, seed);

    write_hashes_parquet::<UInt64Type>(
        output,
        hashes.into_iter().map(|hash| hash.into_iter()),
        dataset.ids().into_iter().map(|id| id.value()),
    )?;

    Ok(())
}
