use super::traits::MultiTrajectoryLsh;
use super::traits::TrajectoryLsh;
use super::Resolution;
use crate::dyft::VCodeArray;
use crate::dyft::VCodeTools;
use crate::trajectory::Trajectory;
use get_size::GetSize;
use itertools::Itertools;
use num_traits::WrappingAdd;
use num_traits::WrappingMul;
use num_traits::Zero;
use rand::Fill;
use rand::Rng;
use std::ops::Shr;

pub struct TensoredMultiHash<H>
where
    H: TrajectoryLsh,
{
    m_left_fns: Vec<Vec<H>>,
    m_right_fns: Vec<Vec<H>>,
    m_rand_coefficients: Vec<H::Hash>,
    m_hashes: usize,
    m_concatenations: usize,
    m_repititions: usize,
}

impl<H> TensoredMultiHash<H>
where
    H: TrajectoryLsh,
{
    fn repititions(l: f64) -> usize {
        l.sqrt().ceil() as usize
    }

    fn concatenations_left(k: f64) -> usize {
        (k / 2.0).ceil() as usize
    }

    fn concatenations_right(k: f64) -> usize {
        (k / 2.0).floor() as usize
    }
}

impl<H> TensoredMultiHash<H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
{
    #[inline]
    fn hash_functions(
        l: usize,
        k: usize,
        resolution: Resolution,
        max_curve_length: usize,
        rnd: &mut impl rand::Rng,
    ) -> Vec<Vec<H>> {
        Vec::from_iter((0..l).map(|_| {
            Vec::from_iter((0..k).map(|_| TrajectoryLsh::init(resolution, max_curve_length, rnd)))
        }))
    }

    #[inline]
    fn k_coefficients(k: usize, rng: &mut impl Rng) -> Vec<H::Hash> {
        let mut coefficients = vec![H::Hash::zero(); k];
        rng.fill(&mut coefficients[..]);
        coefficients
    }
}

impl<'a, H> TensoredMultiHash<H>
where
    H: TrajectoryLsh,
{
    pub fn hash_length(&self) -> usize {
        self.m_hashes
    }

    pub fn concatenations(&self) -> usize {
        self.m_concatenations
    }
}

impl<'a, H> MultiTrajectoryLsh<'a> for TensoredMultiHash<H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
    H: 'a,
{
    type Hasher = H;

    fn init(
        l: usize,
        k: usize,
        resolution: f64,
        max_curve_length: usize,
        rng: &mut impl Rng,
    ) -> Self {
        let m_repititions = Self::repititions(l as f64);
        let m_left_fns = Self::hash_functions(
            m_repititions,
            Self::concatenations_left(k as f64),
            resolution,
            max_curve_length,
            rng,
        );
        let m_right_fns = Self::hash_functions(
            m_repititions,
            Self::concatenations_right(k as f64),
            resolution,
            max_curve_length,
            rng,
        );
        let m_rand_coefficients = Self::k_coefficients(k, rng);

        TensoredMultiHash::<H> {
            m_left_fns,
            m_right_fns,
            m_rand_coefficients,
            m_hashes: l,
            m_concatenations: k,
            m_repititions,
        }
    }

    fn multi_hash(&'a self, trajectory: &'a Trajectory) -> impl Iterator<Item = H::Hash> + 'a {
        let left = self.m_left_fns.iter().map(Self::hash_fns_iter);
        let left = self.inner_tensored_hash(trajectory, left);
        let right = self.m_right_fns.iter().map(Self::hash_fns_iter);
        let right = self.inner_tensored_hash(trajectory, right);
        self.inner_multi_hash(left, right, self.m_repititions)
            .take(self.m_hashes)
    }
    fn multi_hash_query(
        &'a self,
        trajectory: &'a Trajectory,
    ) -> impl Iterator<Item = H::Hash> + 'a {
        let left = self.m_left_fns.iter().map(Self::query_hash_fns_iter);
        let left = self.inner_tensored_hash(trajectory, left);
        let right = self.m_right_fns.iter().map(Self::query_hash_fns_iter);
        let right = self.inner_tensored_hash(trajectory, right);
        self.inner_multi_hash(left, right, self.m_repititions)
            .take(self.m_hashes)
    }
}

impl<H> TensoredMultiHash<H>
where
    H: TrajectoryLsh,
    H::Hash: VCodeTools,
{
    fn _inner_multi_hash(
        &self,
        left: impl Iterator<Item = H::Hash>,
        right: impl Iterator<Item = H::Hash> + Clone,
    ) -> impl Iterator<Item = H::Hash> {
        let coeff_0 = self.m_rand_coefficients[0];
        let coeff_1 = self.m_rand_coefficients[1];
        right
            .cycle()
            .chunks(self.m_right_fns.len())
            .into_iter()
            .zip(left)
            .flat_map(|(r, l)| {
                r.map(move |r| {
                    l.wrapping_mul(&coeff_0)
                        .wrapping_add(&r.wrapping_mul(&coeff_1))
                        .shr(H::Hash::N_DIM / 2)
                })
            })
            .take(self.m_hashes)
            .collect::<Vec<H::Hash>>()
            .into_iter()
    }

    fn inner_multi_hash<'a>(
        &self,
        left: impl Iterator<Item = H::Hash> + 'a,
        right: impl Iterator<Item = H::Hash> + Clone + 'a,
        count: usize,
    ) -> impl Iterator<Item = H::Hash> + 'a
    where
        H::Hash: Clone + 'a,
    {
        let coeff_0 = self.m_rand_coefficients[0];
        let coeff_1 = self.m_rand_coefficients[1];
        left.take(count).flat_map(move |l| {
            right.clone().take(count).map(move |r| {
                l.wrapping_mul(&coeff_0)
                    .wrapping_add(&r.wrapping_mul(&coeff_1))
                    .shr(H::Hash::N_DIM / 2)
            })
        })
    }

    fn hash_fns_iter<'a>(
        functions: &'a Vec<H>,
    ) -> impl Iterator<Item = impl for<'b> Fn(&'b Trajectory) -> H::Hash + Clone + 'a> + Clone + 'a
    {
        functions.iter().map(|f: &H| |t: &Trajectory| f.hash(t))
    }
    fn query_hash_fns_iter<'a>(
        functions: &'a Vec<H>,
    ) -> impl Iterator<Item = impl for<'b> Fn(&'b Trajectory) -> H::Hash + Clone + 'a> + Clone + 'a
    {
        functions
            .iter()
            .map(|f: &H| |t: &Trajectory| f.hash_query(t))
    }

    fn inner_tensored_hash<'a>(
        &'a self,
        trajectory: &'a Trajectory,
        functions: impl Iterator<Item = impl Iterator<Item = impl Fn(&Trajectory) -> H::Hash>>
            + Clone
            + 'a,
    ) -> impl Iterator<Item = H::Hash> + Clone + 'a {
        functions.map(move |hash_fns| {
            hash_fns
                .zip(self.m_rand_coefficients.iter())
                .fold(H::Hash::zero(), |acc, (hash, coeff)| {
                    acc.wrapping_add(&hash(trajectory).wrapping_mul(&coeff))
                })
                .shr(H::Hash::N_DIM / 2)
        })
    }
}

impl<H> TensoredMultiHash<H>
where
    H: TrajectoryLsh,
    [H::Hash]: Fill,
{
    pub fn hash_to_vcodes(&self, dataset: &[Trajectory], bits: usize) -> VCodeArray<H::Hash> {
        VCodeArray::<H::Hash>::from_hashes(dataset.iter().flat_map(|t| self.multi_hash(t)), bits)
    }

    pub fn hash_query_to_vcodes(&self, dataset: &[Trajectory], bits: usize) -> VCodeArray<H::Hash> {
        VCodeArray::<H::Hash>::from_hashes(
            dataset.iter().flat_map(|t| self.multi_hash_query(t)),
            bits,
        )
    }
}

impl<H> GetSize for TensoredMultiHash<H>
where
    H: TrajectoryLsh + GetSize,
    H::Hash: GetSize,
{
    fn get_heap_size(&self) -> usize {
        self.m_left_fns.get_heap_size()
            // .iter()
            // .map(|v| v.get_heap_size())
            // .sum::<usize>()
            + 
                // .m_right_fns
                // .iter()
                // .map(|v| v.get_heap_size())
                // .sum::<usize>()
                self.m_right_fns.get_heap_size()
            + self.m_rand_coefficients.get_heap_size()
            + self.m_hashes.get_heap_size()
            + self.m_concatenations.get_heap_size()
    }
}
