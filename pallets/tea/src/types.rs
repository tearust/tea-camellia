use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::prelude::*;

/// Tea public key generated from the TEA secure module (Tpm, Aws Nitro etc.) used to identify
/// the TEA node.
pub type TeaPubKey = [u8; 32];

/// Peer ID is from IPFS used to identify an IPFS node.
pub type PeerId = Vec<u8>;

/// Cid is from IPFS used to identify an persistent data.
pub type Cid = Vec<u8>;

/// Signature data signed by supported types of keys.
pub type Signature = Vec<u8>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub enum NodeStatus {
	Pending,
	Active,
	Inactive,
	Invalid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct Node<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	pub tea_id: TeaPubKey,
	pub ephemeral_id: TeaPubKey,
	pub profile_cid: Cid,
	pub peer_id: PeerId,
	pub create_time: BlockNumber,
	pub update_time: BlockNumber,
	pub ra_nodes: Vec<(TeaPubKey, bool)>,
	pub status: NodeStatus,
}

impl<BlockNumber> Node<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	pub fn is_active(&self) -> bool {
		self.status == NodeStatus::Active
	}
}

impl<BlockNumber> Default for Node<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	fn default() -> Self {
		Node {
			tea_id: [0u8; 32],
			ephemeral_id: [0u8; 32],
			profile_cid: Vec::new(),
			peer_id: Vec::new(),
			create_time: BlockNumber::default(),
			update_time: BlockNumber::default(),
			ra_nodes: Vec::new(),
			status: NodeStatus::Pending,
		}
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RaResult {
	pub tea_id: TeaPubKey,
	pub target_tea_id: TeaPubKey,
	pub is_pass: bool,
	/// None if target status not changed
	pub target_status: Option<NodeStatus>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RuntimeActivity<BlockNumber> {
	pub tea_id: TeaPubKey,
	pub cid: Option<Cid>,
	pub ephemeral_id: TeaPubKey,
	pub update_height: BlockNumber,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct ReportEvidence<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	pub height: BlockNumber,
	pub reporter: TeaPubKey,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct TipsEvidence<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	pub height: BlockNumber,
	pub target: TeaPubKey,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct OfflineEvidence<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned,
{
	pub height: BlockNumber,
	pub tea_id: TeaPubKey,
}

pub type PcrValue = Vec<u8>;
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct PcrSlots {
	pub slots: Vec<PcrValue>,
	pub description: Vec<u8>,
}
