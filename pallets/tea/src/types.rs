use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// Url is a normal literal string.
pub type Url = Vec<u8>;

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
	BlockNumber: Default,
{
	pub tea_id: TeaPubKey,
	pub ephemeral_id: TeaPubKey,
	pub profile_cid: Cid,
	pub urls: Vec<Url>,
	pub peer_id: PeerId,
	pub create_time: BlockNumber,
	pub update_time: BlockNumber,
	pub ra_nodes: Vec<(TeaPubKey, bool)>,
	pub status: NodeStatus,
}

impl<BlockNumber> Node<BlockNumber>
where
	BlockNumber: Default,
{
	pub fn is_active(&self) -> bool {
		self.status == NodeStatus::Active
	}
}

impl<BlockNumber> Default for Node<BlockNumber>
where
	BlockNumber: Default,
{
	fn default() -> Self {
		Node {
			tea_id: [0u8; 32],
			ephemeral_id: [0u8; 32],
			profile_cid: Vec::new(),
			urls: Vec::new(),
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
	pub target_status: NodeStatus,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, TypeInfo)]
pub struct RuntimeActivity<BlockNumber> {
	pub tea_id: TeaPubKey,
	pub cid: Option<Cid>,
	pub ephemeral_id: TeaPubKey,
	pub update_height: BlockNumber,
}
