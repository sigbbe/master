use serde::Deserialize;
use serde::Serialize;
use std::cmp::Eq;
use std::fmt::Debug;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::num::Wrapping;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Sub;
use std::ops::SubAssign;
use crate::trajectory::Trajectory;

pub type TrajectoryID<'a> = ID<&'a Trajectory>;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Deserialize, Serialize)]
pub struct ID<T>(Wrapping<i64>, std::marker::PhantomData<T>);

impl<T> ID<T> {
    const INVALID_VALUE: Wrapping<i64> = Wrapping(i64::MAX);

    pub fn new(id: i64) -> Self {
        ID(Wrapping(id), PhantomData)
    }

    pub fn invalid() -> Self {
        ID(Self::INVALID_VALUE, PhantomData)
    }

    pub fn is_valid(&self) -> bool {
        self.0 != Self::INVALID_VALUE
    }

    pub fn invalidate(&mut self) {
        self.0 = Self::INVALID_VALUE;
    }

    pub fn value(&self) -> i64 {
        self.0 .0
    }
}

impl<T> From<ID<T>> for i64 {
    fn from(id: ID<T>) -> Self {
        id.0 .0
    }
}

impl<T> From<i64> for ID<T> {
    fn from(id: i64) -> Self {
        ID(Wrapping(id), PhantomData)
    }
}

impl<T> From<usize> for ID<T> {
    fn from(id: usize) -> Self {
        ID(Wrapping(id as i64), PhantomData)
    }
}

impl<T> From<ID<T>> for usize {
    fn from(id: ID<T>) -> Self {
        { id.0 }.0 as usize
    }
}

impl<T> Add<ID<T>> for ID<T> {
    type Output = ID<T>;

    fn add(self, rhs: ID<T>) -> Self::Output {
        ID::new((self.0 + rhs.0).0)
    }
}

impl<T> Add<i64> for ID<T> {
    type Output = ID<T>;

    fn add(self, rhs: i64) -> Self::Output {
        ID::new((self.0 + Wrapping(rhs)).0)
    }
}

impl<T> Sub<ID<T>> for ID<T> {
    type Output = ID<T>;

    fn sub(self, rhs: ID<T>) -> Self::Output {
        ID::new((self.0 - rhs.0).0)
    }
}

impl<T> Sub<i64> for ID<T> {
    type Output = ID<T>;

    fn sub(self, rhs: i64) -> Self::Output {
        ID::new((self.0 - Wrapping(rhs)).0)
    }
}

impl<T> Div<i64> for ID<T> {
    type Output = ID<T>;

    fn div(self, rhs: i64) -> Self::Output {
        ID::new((self.0 / Wrapping(rhs)).0)
    }
}

impl<T> AddAssign<ID<T>> for ID<T> {
    fn add_assign(&mut self, rhs: ID<T>) {
        self.0 += rhs.0;
    }
}

impl<T> SubAssign<ID<T>> for ID<T> {
    fn sub_assign(&mut self, rhs: ID<T>) {
        self.0 -= rhs.0;
    }
}

impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> PartialEq<i64> for ID<T> {
    fn eq(&self, other: &i64) -> bool {
        self.0 .0 == *other
    }
}

impl<T> Eq for ID<T> {}


// Define custom hash function to be able to use IDs with maps/sets
impl<T> Hash for ID<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0 .0.hash(state);
    }
}
