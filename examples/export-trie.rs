#![allow(non_snake_case)]

use anyhow::Result;
use clap::Parser;
use master::config::*;
use master::dyft::DyFT;
use master::dyft::DyftIndex;
use master::dyft::MartExporter;
use master::lsh::Linear64;
use master::io;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
pub struct ExportConfig {
    #[arg(
        short,
        long,
        help = "Path to the dataset (required) in parquet format. See the README for more information."
    )]
    pub datapath: PathBuf,

    #[arg(
        short,
        long,
        help = "Path to the config file (required), if $MASTER_CONFIG_DIR is set, the path is relative to it"
    )]
    pub config: PathBuf,

    #[arg(short, long, help = "Path to the output file (required)")]
    pub output: PathBuf,

    #[arg(
        short,
        long,
        help = "Number of datapoints to use build the index (required)"
    )]
    pub n: usize,
}

fn main() -> Result<()> {
    let ExportConfig {
        datapath,
        config,
        output,
        n,
    } = ExportConfig::parse();

    let config: IndexConfig<MartConfig> = master_config(config)?;

    let dataset = io::trajectory_dataset(&datapath)?;
    let data = dataset.trajectories();

    // instantiate the index
    let mut dyft = DyftIndex::<Linear64>::new(
        &config, 
        dataset.max_trajectory_length(),
    );
    let vcodes = dyft.hash_dataset(&data);

    println!("Vertical vectors");
    for i in 0..n {
        println!("{:?}", vcodes.access(i));
    }

    dyft.build(&vcodes, n);
    
    println!("Built MartIndex with {} vectors", dyft.size());

    // export the index
    dyft.export().to_file(&output)?;

    Ok(())
}
