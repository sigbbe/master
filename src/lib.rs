#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(adt_const_params)]
#![feature(associated_type_defaults)]
#![feature(lazy_type_alias)]
#![feature(test)]
#![feature(const_trait_impl)]
#![feature(effects)]
#![feature(type_alias_impl_trait)]
#![feature(iter_map_windows)]
#![feature(iter_array_chunks)]
#![feature(step_trait)]
#![feature(slice_flatten)]
#![feature(slice_pattern)]
#![feature(int_roundings)]
#![feature(portable_simd)]
#![feature(inherent_associated_types)]
#![feature(write_all_vectored)]

pub mod dyft;
pub mod id;
pub mod io;
pub mod lsh;
pub mod point;
pub mod trajectory;
pub mod util;
pub mod fresh;
pub mod frechet;
pub mod config;
pub mod params;
pub mod benchmarks;