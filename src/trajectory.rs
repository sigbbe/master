use crate::id::TrajectoryID;
use crate::point::Distance;
use crate::point::PointMatrix;
use geo::BoundingRect;
use geo::Coord;
use geo::LineString;
use geo::Rect;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use std::ops::Index;
use geo::algorithm::FrechetDistance;

pub struct TrajectoryDataset<'a> {
    ids: Vec<TrajectoryID<'a>>,
    data: Vec<Trajectory>,
}

impl<'a> TrajectoryDataset<'a> {
    pub fn new(data: IndexMap<TrajectoryID<'a>, Trajectory>) -> Self {
        let (ids, data) = data.into_iter().unzip();
        Self { ids, data }
    }
}

impl<'a> TrajectoryDataset<'a> {
    pub fn len(&self) -> usize {
        self.ids.len().min(self.data.len())
    }
    pub fn max_trajectory_length(&self) -> usize {
        self.data.iter().map(|t| t.len()).max().unwrap_or(0)
    }
    pub fn ids(&self) -> &[TrajectoryID<'a>] {
        self.ids.as_ref()
    }
    pub fn trajectories(&self) -> &[Trajectory] {
        self.data.as_ref()
    }
    pub fn take(self, n: Option<usize>) -> Self {
        match n {
            Some(n) if n < self.len() => Self {
                ids: self.ids.into_iter().take(n).collect(),
                data: self.data.into_iter().take(n).collect(),
            },
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Trajectory(
    #[serde(
        deserialize_with = "deserialize_points",
        serialize_with = "serialize_point"
    )]
    Vec<PointMatrix>,
);

fn deserialize_points<'de, D>(deserializer: D) -> Result<Vec<PointMatrix>, D::Error>
where
    D: Deserializer<'de>,
{
    let points = Vec::<[Distance; 2]>::deserialize(deserializer)?;
    Ok(points
        .into_iter()
        .map(|[x, y]| PointMatrix::from([x, y]))
        .collect())
}

fn serialize_point<'a, S, T>(trajectory: &'a T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: IntoIterator<Item = PointMatrix> + Clone,
{
    serializer.collect_seq(trajectory.clone().into_iter().map(|p| [p[0], p[1]]))
}

impl BoundingRect<Distance> for Trajectory {
    type Output = Option<Rect<Distance>>;

    fn bounding_rect(&self) -> Self::Output {
        LineString::from(
            self.as_ref()
                .iter()
                .map(|p| [p[0], p[1]])
                .collect::<Vec<_>>(),
        )
        .bounding_rect()
    }
}

impl<T: IntoIterator<Item = [Distance; 2]>> From<T> for Trajectory {
    fn from(value: T) -> Self {
        Trajectory(
            value
                .into_iter()
                .map(|[x, y]| PointMatrix::from([x, y]))
                .collect(),
        )
    }
}

impl Trajectory {
    pub const fn new() -> Self {
        Self(vec![])
    }
    pub fn iter(&self) -> impl Iterator<Item = &PointMatrix> {
        self.0.iter()
    }
    pub fn line_string(&self) -> LineString<f64> {
        LineString::from_iter(self.0.iter().map(|p| Coord { x: p[0], y: p[1] }))
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn line_segment(&self, i: usize) -> (PointMatrix, PointMatrix) {
        let pi = self.0[i];
        let pi1 = self.0[i + 1];
        (pi, pi1)
    }
    pub fn append_point(&mut self, point: PointMatrix) {
        self.0.push(point);
    }

    pub fn frechet_decider(&self, other: &Self, bound: Distance) -> bool {
        // crate::frechet::discrete_frechet_distance_predicate(self, other, bound)
        self.line_string().frechet_distance(&other.line_string()) <= bound
    }
}

impl Index<usize> for Trajectory {
    type Output = PointMatrix;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl AsRef<[PointMatrix]> for Trajectory {
    fn as_ref(&self) -> &[PointMatrix] {
        &self.0
    }
}

pub struct TrajectoryIterator<'a> {
    trajectory: &'a Trajectory,
    index: usize,
}

impl FromIterator<PointMatrix> for Trajectory {
    fn from_iter<T: IntoIterator<Item = PointMatrix>>(iter: T) -> Self {
        Trajectory(iter.into_iter().collect())
    }
}

impl IntoIterator for Trajectory {
    type Item = PointMatrix;
    type IntoIter = TrajectoryIterator<'static>;

    fn into_iter(self) -> Self::IntoIter {
        TrajectoryIterator {
            trajectory: Box::leak(Box::new(self)),
            index: 0,
        }
    }
}

impl<'a> Iterator for TrajectoryIterator<'a> {
    type Item = PointMatrix;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.trajectory.0.get(self.index) {
            self.index += 1;
            Some(*point)
        } else {
            None
        }
    }
}
