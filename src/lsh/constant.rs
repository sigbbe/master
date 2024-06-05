use super::linear::LinearFactorLsh;
use super::traits::TrajectoryLsh;
use super::util::random_pertrubation;
use super::Resolution;
use crate::dyft::VCodeTools;
use crate::point::Distance;
use crate::point::Point;
use crate::point::PointMatrix;
use crate::trajectory::Trajectory;
use get_size::GetSize;
use rand::Fill;
use rand::Rng;

#[derive(GetSize)]
pub struct ConstantFactorLsh<T> {
    inner_lsh: LinearFactorLsh<T>,
    data_perturbations: Vec<Point>,
    query_perturbations: Vec<Point>,
    delta: Distance,
}

fn perturbe<'a>(n: usize, resolution: f64, rng: &'a mut impl Rng) -> impl Iterator<Item = Point> +'a {
    (0..n).map(move |_| random_pertrubation(resolution, rng))
}

impl<T> ConstantFactorLsh<T> {
    fn perturb_trajectory(
        trajectory: &Trajectory,
        coefficients: &[Point],
        delta: Distance,
    ) -> Trajectory {
        let mut last_point = [Resolution::MAX, Resolution::MAX];
        Trajectory::from_iter(trajectory.iter().enumerate().filter_map(|(i, point)| {
            let [x, y] = [
                ((point.x + coefficients[i][0]) / delta).round(),
                ((point.y + coefficients[i][1]) / delta).round(),
            ];
            if x != last_point[0] || y != last_point[1] {
                last_point = [x, y];
                Some(PointMatrix::new(x, y))
            } else {
                None
            }
        }))
    }
}

impl<T> TrajectoryLsh for ConstantFactorLsh<T>
where
    T: VCodeTools,
    [T]: Fill,
{
    type Hash = <LinearFactorLsh<T> as TrajectoryLsh>::Hash;

    fn init(delta: f64, max_len: usize, rng: &mut impl Rng) -> Self {
        let inner_lsh = LinearFactorLsh::init(delta, max_len, rng);
        let data_perturbations = Vec::from_iter(perturbe(max_len * 2, delta, rng));
        let query_perturbations = Vec::from_iter(perturbe(max_len * 2, delta, rng));
        ConstantFactorLsh {
            inner_lsh,
            data_perturbations,
            query_perturbations,
            delta,
        }
    }

    fn hash(&self, trajectory: &Trajectory) -> Self::Hash {
        self.inner_lsh.hash(&Self::perturb_trajectory(
            trajectory,
            &self.data_perturbations,
            self.delta,
        ))
    }

    fn hash_query(&self, trajectory: &Trajectory) -> Self::Hash {
        self.inner_lsh.hash(&Self::perturb_trajectory(
            trajectory,
            &self.data_perturbations,
            self.delta,
        ))
    }
}
