// https://github.com/kampersanda/dyft/blob/master/include/mart_common.hpp

use super::node_types::IntoMartNode;
use crate::dyft::MartPointerOffset;
use get_size::GetSize;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::PartialEq;
use std::convert::TryFrom;
use std::fmt::Debug;

pub const MART_NIL_TYPE: MartNodeTypes = MartNodeTypes::MartNilNode;
pub const MART_NILID: u32 = u32::MAX;
pub const MART_NIL_LABEL: u8 = u8::MAX;

pub const MART_NID_BITS: usize = 32;
pub const MART_NTYPE_BITS: usize = 8;
pub const MART_PTR_SIZE: usize = 5;

pub type RawMartPointer = [u8; MART_PTR_SIZE];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MartInsertFlags {
    MartFound,
    MartInserted,
    MartNeededToExpand,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, GetSize)]
pub enum MartNodeTypes {
    MartLeafNode,
    Mart2Node,
    Mart4Node,
    Mart8Node,
    Mart16Node,
    Mart32Node,
    Mart64Node,
    Mart128Node,
    Mart256Node,
    MartNilNode,
}

impl From<MartNodeTypes> for u8 {
    fn from(value: MartNodeTypes) -> Self {
        match value {
            MartNodeTypes::MartLeafNode => 0,
            MartNodeTypes::Mart2Node => 1,
            MartNodeTypes::Mart4Node => 2,
            MartNodeTypes::Mart8Node => 3,
            MartNodeTypes::Mart16Node => 4,
            MartNodeTypes::Mart32Node => 5,
            MartNodeTypes::Mart64Node => 6,
            MartNodeTypes::Mart128Node => 7,
            MartNodeTypes::Mart256Node => 8,
            MartNodeTypes::MartNilNode => 9,
        }
    }
}

impl From<u8> for MartNodeTypes {
    fn from(value: u8) -> Self {
        match value {
            0 => MartNodeTypes::MartLeafNode,
            1 => MartNodeTypes::Mart2Node,
            2 => MartNodeTypes::Mart4Node,
            3 => MartNodeTypes::Mart8Node,
            4 => MartNodeTypes::Mart16Node,
            5 => MartNodeTypes::Mart32Node,
            6 => MartNodeTypes::Mart64Node,
            7 => MartNodeTypes::Mart128Node,
            8 => MartNodeTypes::Mart256Node,
            _ => MartNodeTypes::MartNilNode,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Ord, Eq, PartialEq, PartialOrd, Serialize, Deserialize, GetSize)]
pub struct MartPointer {
    pub nid: u32,
    pub ntype: MartNodeTypes,
}

impl Default for MartPointer {
    fn default() -> Self {
        Self::nil()
    }
}

impl From<[&u8; MART_PTR_SIZE]> for MartPointer {
    fn from(value: [&u8; MART_PTR_SIZE]) -> Self {
        let nid = u32::from_le_bytes([*value[0], *value[1], *value[2], *value[3]]);
        let ntype = MartNodeTypes::from(*value[4]);
        MartPointer { nid, ntype }
    }
}

impl TryFrom<Vec<&u8>> for MartPointer {
    type Error = MartPointer;

    fn try_from(value: Vec<&u8>) -> Result<Self, Self::Error> {
        if value.len() == MART_PTR_SIZE {
            let nid = u32::from_le_bytes([*value[0], *value[1], *value[2], *value[3]]);
            let ntype = MartNodeTypes::from(*value[4]);
            Ok(MartPointer { nid, ntype })
        } else {
            Err(MartPointer::nil())
        }
    }
}

impl From<&MartPointer> for RawMartPointer {
    fn from(value: &MartPointer) -> Self {
        let nid = value.nid.to_le_bytes();
        let ntype: u8 = value.ntype.into();
        [nid[0], nid[1], nid[2], nid[3], ntype]
    }
}

impl From<&RawMartPointer> for MartPointer {
    fn from(value: &RawMartPointer) -> Self {
        let nid = u32::from_le_bytes([value[0], value[1], value[2], value[3]]);
        let ntype = MartNodeTypes::from(value[4]);
        MartPointer { nid, ntype }
    }
}

impl TryFrom<&[u8]> for MartPointer {
    type Error = MartPointer;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match <&[u8] as TryInto<&RawMartPointer>>::try_into(value) {
            Ok(raw_ptr) => Ok(MartPointer::from(raw_ptr)),
            Err(_) => Err(MartPointer::nil()),
        }
    }
}

impl<P> From<&P> for MartPointer
where
    P: IntoMartNode,
{
    fn from(value: &P) -> Self {
        Self {
            nid: value.nid(),
            ntype: value.ntype(),
        }
    }
}

impl MartPointer {
    pub fn new(nid: u32, ntype: MartNodeTypes) -> Self {
        Self { nid, ntype }
    }
    pub fn leaf(nid: u32) -> Self {
        MartPointer {
            nid,
            ntype: MartNodeTypes::MartLeafNode,
        }
    }
    pub fn is_leaf(&self) -> bool {
        self.ntype == MartNodeTypes::MartLeafNode
    }
    pub const fn nil() -> Self {
        MartPointer {
            nid: MART_NILID,
            ntype: MartNodeTypes::MartNilNode,
        }
    }
    pub const fn nil_raw() -> RawMartPointer {
        [0xFF; MART_PTR_SIZE]
    }

    pub fn is_null_ptr(&self) -> bool {
        self.nid == MART_NILID || self.ntype == MartNodeTypes::MartNilNode
    }

    pub fn nid(&self) -> u32 {
        self.nid
    }
    pub fn nid_idx(&self) -> usize {
        self.nid
            .try_into()
            .expect("overflow in MartPointer::nid_idx")
    }

    pub fn nid_mut(&mut self) -> &mut u32 {
        &mut self.nid
    }

    pub fn ntype(&self) -> MartNodeTypes {
        self.ntype
    }

    pub fn ntype_mut(&mut self) -> &mut MartNodeTypes {
        &mut self.ntype
    }
}

#[derive(Debug)]
pub struct MartCursor {
    pub offset: MartPointerOffset,
    pub pptr: MartPointer, // src pointer
    pub nptr: MartPointer, // dst pointer
}

impl MartCursor {
    pub fn new(offset: MartPointerOffset, pptr: MartPointer, nptr: MartPointer) -> Self {
        Self { offset, pptr, nptr }
    }
    pub fn from_next(nptr: &MartPointer) -> Self {
        Self {
            offset: MART_NILID as usize,
            pptr: MartPointer::nil(),
            nptr: MartPointer {
                nid: nptr.nid(),
                ntype: nptr.ntype(),
            },
        }
    }

    pub fn offset(&self) -> MartPointerOffset {
        self.offset
    }

    pub fn pptr(&self) -> &MartPointer {
        &self.pptr
    }

    pub fn nptr(&self) -> &MartPointer {
        &self.nptr
    }

    pub fn nptr_mut(&mut self) -> &mut MartPointer {
        &mut self.nptr
    }

    pub fn ntype(&self) -> MartNodeTypes {
        self.nptr.ntype
    }

    pub fn ptype(&self) -> MartNodeTypes {
        self.pptr.ntype
    }

    pub fn update(&mut self, offset: MartPointerOffset, nptr: &MartPointer) {
        self.offset = offset;
        self.pptr = MartPointer::new(self.nptr.nid(), self.nptr.ntype());
        self.nptr = MartPointer::new(nptr.nid(), nptr.ntype());
    }

    pub fn is_leaf(&self) -> bool {
        self.nptr.ntype == MartNodeTypes::MartLeafNode
    }
}

#[derive(Debug, Deserialize, Serialize, GetSize)]
pub struct MartEdge {
    pub label: u8,
    pub ptr: MartPointer,
}

impl MartEdge {
    pub fn label_idx(&self) -> usize {
        self.label.into()
    }
    pub fn is_leaf(&self) -> bool {
        self.ptr.is_leaf()
    }
}
