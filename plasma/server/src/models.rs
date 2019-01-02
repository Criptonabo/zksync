use web3::types::{U256, U128, H256};
use plasma::models::*;
use std::sync::mpsc::{Sender};
use std::sync::Arc;
use crate::schema::*;
use fnv::{FnvHashSet};

use super::mem_pool::TxQueue;

type TransferBlockResult = Result<(BlockAssemblyResponse, BlockNumber), BlockAssemblyResponse>;

pub enum StateProcessingRequest{
    CreateTransferBlock(TxQueue, bool, Sender<(TxQueue, TransferBlockResult)>),
    ApplyBlock(Block),
    GetPubKey(u32, Sender<Option<PublicKey>>),   // return public key if found
    GetLatestState(u32, Sender<Option<Account>>), // return account state
}

#[derive(Debug, Clone)]
pub struct InPoolTransaction{
    pub timestamp: std::time::Instant,
    pub lifetime: std::time::Duration,
    pub transaction: TransferTx,
}

pub enum IndividualTransactionResponse{
    Accepted(Vec<InPoolTransaction>),
    TemporaryRejected(Vec<InPoolTransaction>),
    RejectedCompletely(Vec<InPoolTransaction>),
}

pub enum TransactionPickerResponse{
    Included(InPoolTransaction),
    ValidButNotIncluded(InPoolTransaction),
    TemporaryRejected(InPoolTransaction),
    RejectedCompletely(InPoolTransaction),
}

pub struct BlockAssemblyResponse{
    pub included: Vec<InPoolTransaction>,
    pub valid_but_not_included: Vec<InPoolTransaction>,
    pub temporary_rejected: Vec<InPoolTransaction>,
    pub completely_rejected: Vec<InPoolTransaction>,
    pub affected_accounts: FnvHashSet<u32>,
}

pub type EncodedProof = [U256; 8];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EthBlockData {
    Transfer{
        total_fees:     U128,

        #[serde(with = "serde_bytes")]
        public_data:    Vec<u8>,
    },
    Deposit{
        batch_number:   BatchNumber,
    },
    Exit{
        batch_number:   BatchNumber,

        #[serde(with = "serde_bytes")]
        public_data:    Vec<u8>,
    },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    Commit{
        new_root:   H256,
        block:      Option<Block>,
    },
    Verify{
        proof:      EncodedProof, 
    },
}

impl std::string::ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::Commit{new_root: _, block: _}   => "Commit".to_owned(),
            Action::Verify{proof: _}                => "Verify".to_owned(),
        }
    }
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub action:             Action,
    pub block_number:       u32, 
    pub block_data:         EthBlockData,
    pub accounts_updated:   AccountMap,
}

#[derive(Debug, Queryable, QueryableByName)]
#[table_name="operations"]
pub struct StoredOperation {
    pub id:             i32,
    pub data:           serde_json::Value,
    pub addr:           String,
    pub nonce:          i32,
    pub block_number:   i32,
    pub action_type:    String,
    pub created_at:     std::time::SystemTime,
}