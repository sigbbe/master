use crate::dyft::*;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::mem::size_of;
use std::path::Path;

const SIZE_LIMIT: usize = 1 << 24;

pub trait LoadVCodesFromBin<T: VCodeTools> {
    fn load_vcodes_from_bin<P>(path: P) -> Result<VCodeArray<T>>
    where
        P: AsRef<Path>;
}

impl<T> LoadVCodesFromBin<T> for T
where
    T: VCodeTools,
{
    fn load_vcodes_from_bin<P>(path: P) -> Result<VCodeArray<T>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let bytes: usize = file.metadata()?.len().try_into()?;
        let mut buf = Vec::<u8>::with_capacity(bytes);
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut buf)?;
        let vcodes = buf
            .chunks(size_of::<u64>())
            .map(|w| w.try_into().expect("slice with incorrect length"))
            .map(|w| u64::from_le_bytes(w))
            .filter_map(|w| T::from_u64(w))
            .collect::<Vec<T>>();

        Ok(VCodeArray::<T>::new(&vcodes, 1))
    }
}

pub trait LoadVCodesFromBVecs<T: VCodeTools>
where
    [(); T::N_DIM]:,
{
    fn load_vcodes_from_bvecs(path: impl AsRef<Path>, bits: usize) -> Result<VCodeArray<T>>;
}

impl<T: VCodeTools> LoadVCodesFromBVecs<T> for T
where
    [(); T::N_DIM]:,
{
    fn load_vcodes_from_bvecs(path: impl AsRef<Path>, bits: usize) -> Result<VCodeArray<T>> {
        let file = File::open(path)?;
        let size = file.metadata()?.len() as usize / T::N_DIM;
        assert!(size < SIZE_LIMIT, "size must be less than {SIZE_LIMIT}");

        let mut reader = BufReader::new(file);
        let mut vcodes = VCodeArray::<T>::empty(bits);
        let mut m_dim = [0u8; 4];

        while let Ok(()) = reader.read_exact(&mut m_dim) {
            let len: usize = u32::from_le_bytes(m_dim).try_into().unwrap();
            assert!(len >= T::N_DIM, "Dimensions must be greater than {}", T::N_DIM);
            let mut code = [0u8; T::N_DIM];
            reader.read_exact(&mut code)?;
            vcodes.append(&code);
        }
        Ok(vcodes)
    }
}
