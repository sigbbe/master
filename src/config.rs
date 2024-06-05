use crate::point::Distance;
use anyhow::Result;
use clap::Args;
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;
use std::env::var;
use std::fs::ReadDir;
use std::path::Path;
use std::path::PathBuf;

pub const MASTER_DATA_DIR: &str = "MASTER_DATA_DIR";
pub const MASTER_QUERY_DIR: &str = "MASTER_QUERY_DIR";
pub const MASTER_RESULT_DIR: &str = "MASTER_RESULT_DIR";
pub const MASTER_CONFIG_DIR: &str = "MASTER_CONFIG_DIR";

pub fn master_data_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(var(MASTER_DATA_DIR)?))
}

pub fn master_query_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(var(MASTER_QUERY_DIR)?))
}

pub fn master_config_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(var(MASTER_CONFIG_DIR)?))
}

pub fn master_result_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(var(MASTER_RESULT_DIR)?))
}

pub fn map_master_path(path: impl AsRef<Path>, master_path: Result<PathBuf>) -> PathBuf {
    master_path
        .map(|p| p.join(path.as_ref()))
        .unwrap_or(path.as_ref().to_path_buf())
}

pub fn master_config<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let config = std::fs::read_to_string(map_master_path(path, master_config_dir()))?;
    Ok(toml::from_str(&config)?)
}

pub fn list_datasets() -> Result<ReadDir> {
    Ok(std::fs::read_dir(master_data_dir()?)?.into_iter())
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
pub struct LshConfig {
    #[arg(long = "k", help = "The number of hash functions to concatenate")]
    pub k: usize,

    #[arg(long = "l", help = "The length of the hash function")]
    pub l: usize,

    #[arg(long, help = "The resolution of the hash function")]
    pub resolution: Distance,

    #[arg(long, help = "The seed for the random number generator")]
    pub seed: u64,
}

impl Default for LshConfig {
    fn default() -> Self {
        LshConfig {
            k: 2,
            l: 8,
            resolution: 0.0,
            seed: 0,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
pub struct IndexConfig<T>
where
    T: Args,
{
    #[command(flatten)]
    pub lsh: LshConfig,

    #[command(flatten)]
    pub index: T,
}

impl Default for IndexConfig<MartConfig> {
    fn default() -> Self {
        IndexConfig {
            lsh: Default::default(),
            index: MartConfig {
                bits: 8,
                splitthreshold: None,
                in_weight: Some(1.0),
                radius: 8,
                errors: 8,
                distance: None,
            },
        }
    }
}

impl Default for IndexConfig<FreshConfig> {
    fn default() -> Self {
        Self {
            lsh: Default::default(),
            index: FreshConfig {
                distance: None,
                verify_fraction: None,
            },
        }
    }
}

impl IndexConfig<MartConfig> {
    pub fn with_bits(self, bits: usize) -> Self {
        IndexConfig {
            index: MartConfig { bits, ..self.index },
            ..self
        }
    }

    pub fn with_splitthreshold(self, splitthreshold: u32) -> Self {
        IndexConfig {
            index: MartConfig {
                splitthreshold: Some(splitthreshold),
                ..self.index
            },
            ..self
        }
    }

    pub fn with_in_weight(self, in_weight: f32) -> Self {
        IndexConfig {
            index: MartConfig {
                in_weight: Some(in_weight),
                ..self.index
            },
            ..self
        }
    }

    pub fn with_radius(self, radius: u32) -> Self {
        IndexConfig {
            index: MartConfig {
                radius,
                ..self.index
            },
            ..self
        }
    }

    pub fn with_errors(self, errors: u8) -> Self {
        IndexConfig {
            index: MartConfig {
                errors,
                ..self.index
            },
            ..self
        }
    }

    pub fn with_distance(self, distance: Distance) -> Self {
        IndexConfig {
            index: MartConfig {
                distance: Some(distance),
                ..self.index
            },
            ..self
        }
    }
}

impl<T> IndexConfig<T>
where
    T: Args,
{
    pub fn k(self, k: usize) -> Self {
        IndexConfig {
            lsh: LshConfig { k, ..self.lsh },
            index: self.index,
        }
    }

    pub fn l(self, l: usize) -> Self {
        IndexConfig {
            lsh: LshConfig { l, ..self.lsh },
            index: self.index,
        }
    }

    pub fn resolution(self, resolution: Distance) -> Self {
        IndexConfig {
            lsh: LshConfig {
                resolution,
                ..self.lsh
            },
            index: self.index,
        }
    }
}

impl<T> IndexConfig<T>
where
    T: Args,
{
    pub fn lsh_params(&self) -> &LshConfig {
        &self.lsh
    }

    pub fn index_params(&self) -> &T {
        &self.index
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
pub struct MartConfig {
    #[arg(short, long, help = "The number of bits in each vertical vector code")]
    pub bits: usize,

    #[arg(
        long,
        help = "The split threshold for the MART index, if not set, the optimal split threshold is used"
    )]
    pub splitthreshold: Option<u32>,

    #[arg(
        long,
        help = "Weight factor for the split threshold, if not set, 1.0 is used"
    )]
    pub in_weight: Option<f32>,

    #[arg(
        long,
        help = "Distance threshold for the hamming distance verification"
    )]
    pub radius: u32,

    #[arg(
        long,
        help = "The number of errors allowed when traversing the MART trie index", 
        // value_parse = "MartConfig::parse_errors"
    )]
    pub errors: u8,

    #[arg(
        long,
        help = "Distance threshold for discrete fréchet distance verification"
    )]
    pub distance: Option<Distance>,
}

impl MartConfig {
    pub fn parse_errors(s: &str) -> Result<u8> {
        let errors = s.parse()?;
        if errors < 17 {
            Ok(errors)
        } else {
            Err(anyhow::anyhow!("errors must be less than 17"))
        }
    }

}

#[derive(Clone, Debug, Deserialize, Serialize, Parser)]
pub struct FreshConfig {
    #[arg(
        long,
        help = "Distance threshold for discrete fréchet distance verification"
    )]
    pub distance: Option<Distance>,

    #[arg(long, help = "The fraction of non-zero scored candidates to verify")]
    pub verify_fraction: Option<Distance>,
}

pub struct FreshPartialVerification {
    pub distance: Distance,
    pub verify_fraction: Distance,
}

impl FreshConfig {
    pub fn partial_verification(&self) -> Option<FreshPartialVerification> {
        match (self.distance, self.verify_fraction) {
            (Some(distance), Some(verify_fraction))
                if verify_fraction <= 1.0 && verify_fraction > 0.0 =>
            {
                Some(FreshPartialVerification {
                    distance,
                    verify_fraction,
                })
            }
            _ => None,
        }
    }
}

impl IndexConfig<FreshConfig> {
    pub fn verify_fraction(self, verify_fraction: Distance) -> Self {
        IndexConfig {
            index: FreshConfig {
                verify_fraction: Some(verify_fraction),
                ..self.index
            },
            ..self
        }
    }
    pub fn distance(self, distance: Distance) -> Self {
        IndexConfig {
            index: FreshConfig {
                distance: Some(distance),
                ..self.index
            },
            ..self
        }
    }
}
