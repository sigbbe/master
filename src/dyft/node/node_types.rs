use super::common::MartPointer;
use super::common::MART_NILID;
use super::MartNodeTypes;
use get_size::GetSize;

// a macro that defines the marker types for MartNodeType
macro_rules! define_mart_node_types {
    (@step $idx:expr, $name:ident, $n:expr) => {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Default, GetSize)]
        pub struct $name(u32);

        impl IntoMartNode for $name {
            const BYTES: usize = $n;
            const TYPE_ID: MartNodeTypes = match $n {
                1 => MartNodeTypes::MartLeafNode,
                2 => MartNodeTypes::Mart2Node,
                4 => MartNodeTypes::Mart4Node,
                8 => MartNodeTypes::Mart8Node,
                16 => MartNodeTypes::Mart16Node,
                32 => MartNodeTypes::Mart32Node,
                64 => MartNodeTypes::Mart64Node,
                128 => MartNodeTypes::Mart128Node,
                256 => MartNodeTypes::Mart256Node,
                _ => unreachable!(),
            };

            fn create(nid: u32) -> Self {
                Self(nid)
            }

            fn nid(&self) -> u32 {
                self.0
            }
        }
    };

    (@step $idx:expr, $name:ident = $bytes:expr) => {
        define_mart_node_types!(@step $idx, $name, $bytes);
    };

    (@step $idx:expr, $name:ident = $bytes:expr, $($names_tail:ident = $bytes_tail:expr),*) => {
        define_mart_node_types!(@step $idx, $name, $bytes);
        define_mart_node_types!(@step $idx + 1usize, $($names_tail = $bytes_tail),*);
    };

    ($($names:ident = $bytes:expr),+) => {
        define_mart_node_types!(@step 0usize, $($names = $bytes),*);
    };
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=4b287dcbd2034777fd220fa2e58146a3
// pub trait MartNodeType: TryFrom<u8, Error = MartNilType> + Clone {
pub trait IntoMartNode {
    const BYTES: usize;
    const TYPE_ID: MartNodeTypes;

    fn create(nid: u32) -> Self;

    fn ntype(&self) -> MartNodeTypes {
        Self::TYPE_ID
    }

    fn nid(&self) -> u32;

    fn into_mart(&self) -> MartPointer {
        MartPointer::new(self.nid(), Self::TYPE_ID)
    }
}

#[derive(PartialEq, Eq)]
pub struct MartNilType;

impl IntoMartNode for MartNilType {
    const BYTES: usize = 0;
    const TYPE_ID: MartNodeTypes = MartNodeTypes::MartNilNode;

    fn create(_: u32) -> Self {
        Self
    }

    fn nid(&self) -> u32 {
        MART_NILID
    }
}

define_mart_node_types! {
    MartLeaf = 1,
    MartNode2 = 2,
    MartNode4 = 4,
    MartNode8 = 8,
    MartNode16 = 16,
    MartNode32 = 32,
    MartNode64 = 64,
    MartNode128 = 128,
    MartNode256 = 256
}