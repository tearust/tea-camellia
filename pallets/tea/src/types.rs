use codec::{Decode, Encode};
use sp_std::prelude::*;

pub type Url = Vec<u8>;

pub type TeaPubKey = [u8; 32];

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum NodeStatus {
    Pending,
    Active,
    Inactive,
    Invalid,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Node<BlockNumber> {
    pub tea_id: TeaPubKey,
    pub ephemeral_id: TeaPubKey,
    pub profile_cid: Vec<u8>,
    pub urls: Vec<Url>,
    pub peer_id: Vec<u8>,
    pub create_time: BlockNumber,
    pub update_time: BlockNumber,
    pub ra_nodes: Vec<(TeaPubKey, bool)>,
    pub status: NodeStatus,
}
