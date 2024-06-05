use super::Resolution;
use crate::point::Distance;
use crate::point::Point;
use nalgebra::Const;
use nalgebra::IsNotStaticOne;
use nalgebra::RowSVector;
use num_traits::Zero;
use rand::Fill;

#[allow(dead_code)]
fn fill_shift<const D: usize>(rng: &mut impl rand::Rng) -> RowSVector<Distance, D>
where
    Const<D>: IsNotStaticOne,
    [Distance; D]: Fill,
{
    let mut shifts = [0.0; D];
    rng.fill(&mut shifts);
    RowSVector::<Distance, D>::from(shifts)
}

// A. Driemel and F. Silvestri: p. 7
pub fn random_shift_grid(range: Resolution, rng: &mut impl rand::Rng) -> Point {
    [rng.gen::<Distance>() * range, rng.gen::<Distance>() * range]
}

// A. Driemel and F. Silvestri: p. 9
pub fn random_pertrubation(range: Resolution, rng: &mut impl rand::Rng) -> Point {
    let div = range / 2.0;
    [
        (rng.gen::<Distance>() * range) - div,
        (rng.gen::<Distance>() * range) - div,
    ]
}

pub fn random_coefficients<T>(rng: &mut impl rand::Rng, n: usize) -> Vec<T>
where
    T: Zero + Clone,
    [T]: Fill,
{
    let mut coefficients = vec![T::zero(); n * 2];
    rng.fill(&mut coefficients[..]);
    coefficients
}
