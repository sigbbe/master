use crate::dyft::VCodeTools;
use crate::trajectory::Trajectory;

pub trait TrajectoryLsh {
    type Hash: VCodeTools;

    fn init(delta: f64, max_len: usize, rng: &mut impl rand::Rng) -> Self;

    fn hash(&self, trajectory: &Trajectory) -> Self::Hash;

    /// method to be overwritten by asymmetric hashing schemes
    fn hash_query(&self, trajectory: &Trajectory) -> Self::Hash {
        self.hash(trajectory)
    }
}

pub trait MultiTrajectoryLsh<'a> {
    type Hasher: TrajectoryLsh + 'a;

    fn init(l: usize, k: usize, delta: f64, max_len: usize, rng: &mut impl rand::Rng) -> Self;

    fn multi_hash(
        &'a self,
        trajectory: &'a Trajectory,
    ) -> impl Iterator<Item = <Self::Hasher as TrajectoryLsh>::Hash> + 'a;

    /// method to be overwritten by asymmetric hashing schemes
    fn multi_hash_query(
        &'a self,
        trajectory: &'a Trajectory,
    ) -> impl Iterator<Item = <Self::Hasher as TrajectoryLsh>::Hash> + 'a {
        self.multi_hash(trajectory)
    }
}
