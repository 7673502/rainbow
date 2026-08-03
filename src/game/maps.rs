use crate::constants::{MAX_POINTS_VALUE, MAX_RANK};
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RankMap([u8; MAX_RANK]);

impl Default for RankMap {
    fn default() -> Self {
        Self::new()
    }
}

impl RankMap {
    pub fn new() -> Self {
        RankMap([0; MAX_RANK])
    }
}

impl Index<u8> for RankMap {
    type Output = u8;
    fn index(&self, rank: u8) -> &u8 {
        &self.0[(rank - 1) as usize]
    }
}

impl IndexMut<u8> for RankMap {
    fn index_mut(&mut self, rank: u8) -> &mut u8 {
        &mut self.0[(rank - 1) as usize]
    }
}

pub(crate) struct PointMap([u8; MAX_POINTS_VALUE]);

impl PointMap {
    pub fn new() -> Self {
        PointMap([0; MAX_POINTS_VALUE])
    }
}

impl Index<u8> for PointMap {
    type Output = u8;
    fn index(&self, rank: u8) -> &u8 {
        &self.0[(rank - 1) as usize]
    }
}

impl IndexMut<u8> for PointMap {
    fn index_mut(&mut self, rank: u8) -> &mut u8 {
        &mut self.0[(rank - 1) as usize]
    }
}
