use std::marker::PhantomData;
use get_size::GetSize;

use super::IntoMartNode;
use super::MartOffsets;
use super::MART_PTR_SIZE;

/// These structs are used to define the offsets for the different types of nodes
/// in the trie. The offsets are used to calculate the position of the pointers
/// and labels in the node.  

#[derive(GetSize)]
pub struct MartSparseOffsets<T>(PhantomData<T>);

#[derive(GetSize)]
pub struct MartDenseOffsets<T>(PhantomData<T>);

#[derive(GetSize)]
pub struct MartFullOffsets<T>(PhantomData<T>);

impl<T: IntoMartNode> MartOffsets for MartSparseOffsets<T> {
    const K: usize = T::BYTES;
    
    // header, i.e., number of children takes 1 byte
    const PTRS_OFFSET: usize = 1 + T::BYTES;
    // K keys of 1 byte each
    // K pointers of 5 bytes each
    const BYTES: usize = Self::PTRS_OFFSET + (T::BYTES * MART_PTR_SIZE);
}

const MAX_PTRS: usize = 1 << 8;

impl<T: IntoMartNode> MartOffsets for MartDenseOffsets<T> {
    const K: usize = T::BYTES;
    const PTRS_OFFSET: usize = 1 + MAX_PTRS;
    const BYTES: usize = Self::PTRS_OFFSET + (T::BYTES * MART_PTR_SIZE);
}

impl<T: IntoMartNode> MartOffsets for MartFullOffsets<T> {
    const K: usize = T::BYTES;
    const PTRS_OFFSET: usize = 1;
    const BYTES: usize = Self::PTRS_OFFSET + (MAX_PTRS * MART_PTR_SIZE);
}