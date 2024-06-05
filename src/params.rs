use crate::config::master_config;
use crate::config::IndexConfig;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use serde::Deserialize;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
pub struct BuildArgs {
    #[arg(short, long, help = "Path to the dataset")]
    pub dataset: PathBuf,

    #[arg(
        short,
        long = "datapoints",
        help = "Number of trajectories to load (optional)"
    )]
    pub n: Option<usize>,

    #[arg(short, long, help = "Path to the output file (optional)")]
    pub output: Option<PathBuf>,

    #[arg(help = "Number of indexed trajectories for each memory sample (optional)")]
    pub samples: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Parser)]
pub struct QueryArgs {
    #[arg(short, long, help = "Path to the query set")]
    pub queryset: PathBuf,
}

#[derive(Debug, Clone, Parser)]
pub struct BuildWithConfigPath {
    #[arg(short, long, help = "Path to the configuration file")]
    pub config_file: PathBuf,

    #[command(flatten)]
    pub build: BuildArgs,
}

#[derive(Debug, Clone, Parser)]
pub struct QueryWithConfigPath {
    #[command(flatten)]
    pub config: BuildWithConfigPath,

    #[command(flatten)]
    pub query: QueryArgs,
}

#[derive(Debug, Clone, Parser)]
pub enum IndexCommandWithConfigPath {
    #[clap(name = "build", about = "Build the index")]
    Build(BuildWithConfigPath),

    #[command(name = "query", about = "Build and query the index")]
    Query(QueryWithConfigPath),
}

impl IndexCommandWithConfigPath {
    pub fn try_config_with_args<T>(
        iter: impl Iterator<Item = impl Into<OsString> + Clone>,
    ) -> Result<IndexCommandWithArgs<T>>
    where
        T: Args + for<'de> Deserialize<'de>,
    {
        match Self::try_parse_from(iter)? {
            IndexCommandWithConfigPath::Build(b) => {
                Ok(IndexCommandWithArgs::Build(BuildWithArgs::<T> {
                    config: master_config(b.config_file)?,
                    build: b.build,
                }))
            }
            IndexCommandWithConfigPath::Query(q) => {
                Ok(IndexCommandWithArgs::Query(QueryWithArgs {
                    config: master_config(q.config.config_file)?,
                    build: q.config.build,
                    query: q.query,
                }))
            }
        }
    }
}

#[derive(Debug, Clone, Parser)]
pub struct BuildWithArgs<T>
where
    T: Args,
{
    #[command(flatten)]
    pub config: IndexConfig<T>,

    #[command(flatten)]
    pub build: BuildArgs,
}

#[derive(Debug, Clone, Parser)]
pub struct QueryWithArgs<T>
where
    T: Args,
{
    #[command(flatten)]
    pub config: IndexConfig<T>,

    #[command(flatten)]
    pub build: BuildArgs,

    #[command(flatten)]
    pub query: QueryArgs,
}

#[derive(Debug, Clone, Parser)]
pub enum IndexCommandWithArgs<T>
where
    T: Args,
{
    #[clap(name = "build", about = "Build the index")]
    Build(BuildWithArgs<T>),

    #[clap(name = "query", about = "Build and query the index")]
    Query(QueryWithArgs<T>),
}

impl<T> IndexCommandWithArgs<T>
where
    T: Args + for<'de> Deserialize<'de>,
{
    pub fn parse_from_args_or_file() -> Result<Self> {
        let args = std::env::args().collect::<Vec<_>>();
        if let Some(_) = args.iter().find(|&s| s == "-c" || s == "--config-file") {
            IndexCommandWithConfigPath::try_config_with_args::<T>(args.iter())
        } else {
            Self::try_parse_from(args.iter()).map_err(|e| anyhow::anyhow!(e))
        }
    }

    pub fn memory_samples(&self) -> Option<Vec<usize>> {
        match self {
            IndexCommandWithArgs::Build(b) => b.build.samples.clone(),
            IndexCommandWithArgs::Query(q) => q.build.samples.clone(),
        }
    }

    pub fn data_path(&self) -> &PathBuf {
        match self {
            IndexCommandWithArgs::Build(b) => &b.build.dataset,
            IndexCommandWithArgs::Query(q) => &q.build.dataset,
        }
    }

    pub fn query_path(&self) -> Option<&PathBuf> {
        match self {
            IndexCommandWithArgs::Build(_) => None,
            IndexCommandWithArgs::Query(q) => Some(&q.query.queryset),
        }
    }

    pub fn output_path(&self) -> Option<&PathBuf> {
        match self {
            IndexCommandWithArgs::Build(b) => b.build.output.as_ref(),
            IndexCommandWithArgs::Query(q) => q.build.output.as_ref(),
        }
    }

    pub fn index_config(&self) -> &IndexConfig<T> {
        match self {
            IndexCommandWithArgs::Build(b) => &b.config,
            IndexCommandWithArgs::Query(q) => &q.config,
        }
    }

    pub fn n(&self) -> Option<usize> {
        match self {
            IndexCommandWithArgs::Build(b) => b.build.n,
            IndexCommandWithArgs::Query(q) => q.build.n,
        }
    }
}
