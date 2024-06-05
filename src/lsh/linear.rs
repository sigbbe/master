use super::traits::TrajectoryLsh;
use super::util::random_coefficients;
use super::util::random_shift_grid;
use crate::dyft::VCodeTools;
use crate::point::Distance;
use crate::point::Point;
use crate::trajectory::Trajectory;
use get_size::GetSize;
use rand::Fill;

#[derive(GetSize)]
pub struct LinearFactorLsh<T> {
    delta: Distance,
    shift: Point,
    rand_coefficients: Vec<T>,
}

impl<T> TrajectoryLsh for LinearFactorLsh<T>
where
    T: VCodeTools,
    [T]: Fill,
{
    type Hash = T;

    fn init(delta: f64, max_len: usize, rng: &mut impl rand::Rng) -> Self {
        LinearFactorLsh {
            delta,
            shift: random_shift_grid(delta, rng),
            rand_coefficients: random_coefficients::<T>(rng, max_len),
        }
    }

    // https://github.com/Cecca/FRESH/blob/d7740ed59b1566bf77f6a54ed3423e6a3d62e230/core/hash.h#L142C41-L142C47
    fn hash(&self, trajectory: &Trajectory) -> Self::Hash {
        let delta = self.delta;
        let [shift_x, shift_y] = self.shift;
        trajectory
            .iter()
            .fold(
                (T::zero(), 0, [Distance::MAX, Distance::MAX]),
                |(acc, coeff_idx, [last_x, last_y]): (T, usize, Point), &point| {
                    let [x, y] = [
                        ((point.x + shift_x) / delta).round(),
                        ((point.y + shift_y) / delta).round(),
                    ];
                    if x != last_x || y != last_y {
                        let coeff_x =
                            T::wrap_to_t(x).wrapping_mul(&self.rand_coefficients[coeff_idx]);
                        let coeff_y =
                            T::wrap_to_t(y).wrapping_mul(&self.rand_coefficients[coeff_idx + 1]);
                        (
                            acc.wrapping_add(&coeff_x.wrapping_add(&coeff_y)),
                            coeff_idx + 1,
                            [x, y],
                        )
                    } else {
                        (acc, coeff_idx, [last_x, last_y])
                    }
                },
            )
            .0
            .shr(T::N_DIM / 2)
    }
}