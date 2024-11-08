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
trait Wormhole {
    fn verify_vaa(&self, vaa: String) -> u32;
}


// Prepaid gas for a single (not inclusive of recursion) `verify_vaa` call.
const VERIFY_CALL_GAS: Gas = Gas::from_tgas(20);
const DELIVERY_CALL_GAS: Gas = Gas::from_tgas(100);
const CALL_CALL_GAS: Gas = Gas::from_tgas(5);
const CHAIN_ID_MAINNET: u16 = 2;

//#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
//#[serde(crate = "near_sdk::serde")]
//#[near(serializers=[borsh])]
//#[derive(PartialEq, Clone)]
#[derive(Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde", untagged)]
pub struct Call {
    pub contract_id: AccountId,
    pub method_name: String,
    pub args: Vec<u8>,
}

//#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
//#[serde(crate = "near_sdk::serde")]
#[near(serializers=[borsh])]
#[derive(PartialEq, Clone)]
pub struct CallResult {
    pub success: bool,
    pub result: Option<Vec<u8>>,
}

#[near(contract_state)]
pub struct WormholeRelayer {
    owner: AccountId,
    wormhole_core: AccountId,
    foreign_governor_address: Vec<u8>,
    dups: UnorderedSet<Vec<u8>>
}


#[near]
impl WormholeRelayer {
    /// Initializes the contract owned by `owner_id` 
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

    pub fn change_owner(&mut self, new_owner: AccountId) {
        // Check the ownership
        require!(self.owner == env::predecessor_account_id());

        // Check account validity
        require!(env::is_valid_account_id(new_owner.as_bytes()));

        self.owner = new_owner;

        // TODO: event
    }

    // TODO: needs to be payable?
    #[payable]
    pub fn delivery(
        &mut self,
        vaa: String
    ) -> Promise {
        // TODO: Crate hex
        let h = hex::decode(vaa).expect("invalidVaa");
        let vaa = state::ParsedVAA::parse(&h);

        // Check if VAA with this hash was already accepted
        if self.dups.contains(&vaa.hash) {
            env::panic_str("alreadyExecuted");
        }

        let initial_storage_usage = env::storage_usage();
        self.dups.insert(&vaa.hash);
        let storage = env::storage_usage() - initial_storage_usage;
        self.refund_deposit_to_account(storage, 0, env::predecessor_account_id(), true);

        if (vaa.emitter_chain != CHAIN_ID_MAINNET) || (vaa.emitter_address != foreign_governor_address) {
            env::panic_str("InvalidGovernorEmitter");
        }

        let data: &[u8] = &vaa.payload;
        let num_calls = data.get_u8(0);
        let mut calls = Vec::new();
        let mut size_counter: usize = 1;
        for i in 0..num_calls as usize {
            let vec_size = data.get_u8(size_counter);
            size_counter += 1;
            let buf = data.get_bytes(size_counter, vec_size);
            size_counter += vec_size;

            let mut buf_counter = 0;
            let mut field_size = buf.get_u8(0);
            buf_counter += 1;
            let contract_id = buf.get_bytes(buf_counter, field_size).to_string();
            buf_counter += field_size;
            field_size = buf.get_u8(buf_counter);
            buf_counter += 1;
            let method_name = buf.get_bytes(buf_counter, field_size).to_string();
            buf_counter += field_size;
            let args = buf.get_bytes(buf_counter, vec_size - buf_counter).to_vec();

            calls.push(Call(contract_id, method_name, args));
        }
        for i in 0..num_calls as usize {
            calls[i] = data.get_u8(i + 1);
        }

        Promise::new("wormhole.near".parse().unwrap()).function_call(
            "verify_vaa".to_string(),
            vaa.into_bytes(),
            0,                             // attached deposit
            10_000_000_000_000,            // attached gas
        )
        .then(
            Self::ext(env::current_account_id())
                .with_static_gas(5_000_000_000_000) // gas for current
                .on_verify_complete(calls),
        )
    }

    #[private]
    pub fn on_verify_complete(&self, calls: Vec<Call>) -> Vec<CallResult> {
        let mut results = Vec::new();

        // if Verify pass
        if let PromiseResult::Successful(_) = env::promise_result(0) {
            // Если да, выполняем все вызовы из списка calls
            for (i, call) in calls.iter().enumerate() {
                let promise = Promise::new(call.contract_id.clone()).function_call(
                    call.method_name.clone(),
                    call.args.clone(),
                    0,                            // attached deposit
                    10_000_000_000_000,           // attached gas
                );
                env::promise_return(i as u64);
            }

            // Get results
            for i in 0..calls.len() {
                let result = match env::promise_result(i as u64) {
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
            // Verify failed
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
        log!("storage used: {}", storage_used);
        let near_deposit = NearToken::from_yoctonear(service_deposit);
        let mut required_cost = env::storage_byte_cost().saturating_mul(storage_used.into());
        required_cost = required_cost.saturating_add(near_deposit);

        let mut refund = env::attached_deposit();
        // Deposit is added on a balance
        if deposit_in {
            // Required cost must not be bigger than the attached deposit
            require!(required_cost <= refund);
            refund = refund.saturating_sub(required_cost);
        } else {
            // This could be the case if the storage price went up during the lifespan of the service
            require!(required_cost <= env::account_balance());
            refund = refund.saturating_add(required_cost);
        }
        //log!("required cost: {}", required_cost.as_yoctonear());
        log!("refund: {}", refund.as_yoctonear());
        log!("balance: {}", env::account_balance().as_yoctonear());
        if refund.as_yoctonear() > 1 {
            Promise::new(account_id).transfer(refund);
        }
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

// impl Default for AccountId {
//     fn default() -> Self {
//         Self {
//             account_id: "aaa";
//         }
//     }
// }
//
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


