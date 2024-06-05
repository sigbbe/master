// https://github.com/kampersanda/dyft/blob/master/include/vcode_array.hpp
#![allow(dead_code)]

use super::VCodeTools;
use std::fmt::Debug;

#[derive(Debug)]
pub struct VCodeArray<T: VCodeTools> {
    m_bits: usize,
    m_vcodes: Vec<T>,
}

impl<T: VCodeTools> VCodeArray<T> {
    pub fn empty(m_bits: usize) -> Self {
        assert!(
            m_bits >= 1 && m_bits <= 8,
            "bits must be in [1, 8]: {}",
            m_bits
        );
        Self {
            m_bits,
            m_vcodes: Vec::new(),
        }
    }

    pub fn new(m_vcodes: &[T], m_bits: usize) -> Self {
        assert!(
            m_bits >= 1 && m_bits <= 8,
            "bits must be in [1, 8]: {}",
            m_bits
        );
        Self {
            m_bits,
            m_vcodes: m_vcodes.to_vec(),
        }
    }

    pub fn append(&mut self, code: &[u8]) {
        self.m_vcodes.extend(T::to_vints(code, self.m_bits));
    }

    pub fn access(&self, id: usize) -> &[T] {
        assert!(id < self.size(), "id must be less than size");
        let idx = id * self.m_bits;
        let idx = idx..idx + self.m_bits;
        &self.m_vcodes[idx]
    }
    pub fn vcodes(&self) -> &[T] {
        &self.m_vcodes
    }

    pub fn size(&self) -> usize {
        self.m_vcodes.len() / self.m_bits
    }

    pub fn bits(&self) -> usize {
        self.m_bits
    }

    pub fn verify_candidate_predicate(&self, candidate: usize, query: &[T], radius: u32) -> bool {
        T::hamdist_radius(self.access(candidate), query, self.m_bits, radius) <= radius
    }

    pub fn hamdist_radius(&self, candidate: usize, query: &[T], radius: u32) -> u32 {
        T::hamdist_radius(self.access(candidate), query, self.m_bits, radius)
    }

    pub fn iter(&self) -> impl Iterator<Item = &[T]> {
        (0..self.size()).map(move |i| self.access(i))
    }

    pub fn linear_search<'a>(
        &'a self,
        other: &'a Self,
        radius: u32,
    ) -> impl Iterator<Item = (usize, usize)> + 'a {
        other.iter().enumerate().flat_map(move |(q, query)| {
            (0..self.size())
                .filter(move |&candidate| self.verify_candidate_predicate(candidate, query, radius))
                .map(move |candidate| (q, candidate))
        })
    }
}

impl<'a, T> VCodeArray<T>
where
    T: VCodeTools,
{
    pub fn from_hashes<I>(iter: I, bits: usize) -> Self
    where
        I: Iterator<Item = T>,
    {
        VCodeArray {
            m_bits: bits,
            m_vcodes: Vec::from_iter(iter),
        }
    }
}
