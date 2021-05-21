use codec::{Decode, Encode};
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

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum NodeStatus {
    Pending,
    Active,
    Inactive,
    Invalid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
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
