// #![allow(dead_code)]

mod constant;
mod linear;
mod tensored;
mod util;
mod traits;

pub use traits::TrajectoryLsh;
pub use traits::MultiTrajectoryLsh;
pub use linear::LinearFactorLsh;
pub use constant::ConstantFactorLsh;
pub use tensored::TensoredMultiHash;

pub type CurveToIdx<T> = (T, T);

// https://locka99.gitbooks.io/a-guide-to-porting-c-to-rust/content/features_of_rust/types.html
pub type Resolution = f64;
pub type Coefficient = i64;


pub type Constant128 = ConstantFactorLsh<u128>;
pub type Linear128 = LinearFactorLsh<u128>;

pub type Constant64 = ConstantFactorLsh<u64>;
pub type Linear64 = LinearFactorLsh<u64>;

pub type Constant32 = ConstantFactorLsh<u32>;
pub type Linear32 = LinearFactorLsh<u32>;

pub type Constant16 = ConstantFactorLsh<u16>;
pub type Linear16 = LinearFactorLsh<u16>;

pub type Constant8 = ConstantFactorLsh<u8>;
pub type Linear8 = LinearFactorLsh<u8>;