use crate::dyft::MartNodeId;
use crate::dyft::MartPointer;
use crate::dyft::MartNodeTypes;

// For query processing
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StateType {
    nptr: MartPointer,
    bpos: usize,
    dist: u8,
}

impl StateType {

	pub fn next(&self, nid: MartNodeId, ntype: MartNodeTypes, dist: u8) -> StateType {
		StateType {
			nptr: MartPointer::new(nid, ntype),
			bpos: self.bpos + 1,
			dist: self.dist + dist,
		
		}
	}
	
    pub fn new(nptr: MartPointer, bpos: usize, dist: u8) -> Self {
        Self { nptr, bpos, dist }
    }

    pub fn pos(&self) -> usize {
        self.bpos
    }

	pub fn pos_mut(&mut self) -> &mut usize {
		&mut self.bpos
	}

	pub fn ptr(&self) -> &MartPointer {
		&self.nptr
	}

	pub fn ptr_mut(&mut self) -> &mut MartPointer {
		&mut self.nptr
	}

    pub fn dist(&self) -> u8 {
        self.dist
    }

	pub fn ntype(&self) -> MartNodeTypes {
		self.nptr.ntype
	}

	pub fn nid(&self) -> MartNodeId {
		self.nptr.nid
	}
}