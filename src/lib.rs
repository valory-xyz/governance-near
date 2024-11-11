use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, TokenMetadata, NonFungibleTokenMetadataProvider, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::json_types::{Base64VecU8, U128, Base58PublicKey};
use near_sdk::serde_json::json;
use near_sdk::collections::LazyOption;
use near_sdk::collections::UnorderedSet;
use near_sdk::{
    env, near, require, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, StorageUsage, Gas,
    IntoStorageKey, PromiseError, PromiseResult, NearToken, log
};
use near_sdk::near_bindgen;
use near_sdk::store::{LookupMap, Vector};
use near_sdk::ext_contract;

pub mod byte_utils;
pub mod state;

use crate::byte_utils::{
    get_string_from_32,
    ByteUtils,
};

// Wormhole Core interface
#[ext_contract(wormhole)]
pub trait Wormhole {
    fn verify_vaa(&self, vaa: String) -> u32;
}

// Prepaid gas for a single (not inclusive of recursion) `verify_vaa` call.
const VERIFY_CALL_GAS: Gas = Gas::from_tgas(20);
const DELIVERY_CALL_GAS: Gas = Gas::from_tgas(100);
const CALL_CALL_GAS: Gas = Gas::from_tgas(5);
const CHAIN_ID_MAINNET: u16 = 2;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Call {
    pub contract_id: AccountId,
    pub method_name: String,
    pub args: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CallResult {
    pub success: bool,
    pub result: Option<Vec<u8>>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct WormholeRelayer {
    owner: AccountId,
    wormhole_core: AccountId,
    foreign_governor_address: Vec<u8>,
    dups: UnorderedSet<Vec<u8>>,
}

impl Default for WormholeRelayer {
    fn default() -> Self {
        Self {
            owner: "".parse().unwrap(),
            wormhole_core: "".parse().unwrap(),
            foreign_governor_address: Vec::new(),
            dups: UnorderedSet::new(b"d".to_vec()),
        }
    }
}

#[near_bindgen]
impl WormholeRelayer {
    #[init]
    pub fn new(owner_id: AccountId, wormhole_core: AccountId, foreign_governor_address: Vec<u8>) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: env::predecessor_account_id(),
            wormhole_core,
            foreign_governor_address,
            dups: UnorderedSet::new(b"d".to_vec()),
        }
    }

    pub fn delivery(&mut self, vaa: String) -> Promise {
        let h = hex::decode(vaa).expect("invalidVaa");
        let vaa = state::ParsedVAA::parse(&h);

        if self.dups.contains(&vaa.hash) {
            env::panic_str("alreadyExecuted");
        }

        let initial_storage_usage = env::storage_usage();
        self.dups.insert(&vaa.hash);
        let storage = env::storage_usage() - initial_storage_usage;
        self.refund_deposit_to_account(storage, 0, env::predecessor_account_id(), true);

        if vaa.emitter_chain != CHAIN_ID_MAINNET || vaa.emitter_address != self.foreign_governor_address {
            env::panic_str("InvalidGovernorEmitter");
        }

        let data: &[u8] = &vaa.payload;
        let calls: Vec<Call> = Vec::try_from_slice(data).expect("Failed to deserialize Vec<Call>");

        Promise::new(self.wormhole_core.clone())
            .function_call("verify_vaa".to_string(), vaa.into_bytes(), NearToken::from_yoctonear(0), VERIFY_CALL_GAS)
            .then(Self::ext(env::current_account_id()).with_static_gas(DELIVERY_CALL_GAS).on_verify_complete(calls))
    }

    pub fn change_owner(&mut self, new_owner: AccountId) {
        // Check the ownership
        require!(self.owner == env::predecessor_account_id());

        // Check account validity
        require!(env::is_valid_account_id(new_owner.as_bytes()));

        self.owner = new_owner;

        // TODO: event
    }

    #[private]
    pub fn on_verify_complete(&self, calls: Vec<Call>) -> Vec<CallResult> {
        let mut results = Vec::new();

        if let PromiseResult::Successful(_) = env::promise_result(0) {
            for call in calls.iter() {
                let result = match env::promise_result(0) {
                    PromiseResult::Successful(data) => CallResult {
                        success: true,
                        result: Some(data),
                    },
                    _ => CallResult {
                        success: false,
                        result: None,
                    },
                };
                results.push(result);
            }
        } else {
            for _ in calls.iter() {
                results.push(CallResult {
                    success: false,
                    result: None,
                });
            }
        }

        results
    }

    fn refund_deposit_to_account(&self, storage_used: u64, service_deposit: u128, account_id: AccountId, deposit_in: bool) {
        let near_deposit = NearToken::from_yoctonear(service_deposit);
        let mut required_cost = env::storage_byte_cost().saturating_mul(storage_used.into());
        required_cost = required_cost.saturating_add(near_deposit);

        let mut refund = env::attached_deposit();
        if deposit_in {
            require!(required_cost <= refund);
            refund = refund.saturating_sub(required_cost);
        } else {
            require!(required_cost <= env::account_balance());
            refund = refund.saturating_add(required_cost);
        }
        if refund.as_yoctonear() > 1 {
            Promise::new(account_id).transfer(refund);
        }
    }
}


