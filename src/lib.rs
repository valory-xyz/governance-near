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
    IntoStorageKey, PromiseError, NearToken, log
};
use near_sdk::store::{LookupMap, Vector};
use near_sdk::ext_contract;

pub mod byte_utils;
pub mod state;

#[derive(Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum MultisigMember {
    AccessKey { public_key: Base58PublicKey },
    Account { account_id: AccountId },
}

// Wormhole Core interface
#[ext_contract(wormhole)]
trait Wormhole {
    fn verify_vaa(&self, vaa: String) -> u32;
}



// Prepaid gas for a single (not inclusive of recursion) `verify_vaa` call.
const VERIFY_CALL_GAS: Gas = Gas::from_tgas(20);
const DELIVERY_CALL_GAS: Gas = Gas::from_tgas(100);
const CALL_CALL_GAS: Gas = Gas::from_tgas(5);

#[near(contract_state)]
pub struct WormholeRelayer {
    owner: AccountId,
    wormhole_core: AccountId,
    dups: UnorderedSet<Vec<u8>>
}


#[near]
impl WormholeRelayer {
    /// Initializes the contract owned by `owner_id` 
    #[init]
    pub fn new(owner_id: AccountId, wormhole_core: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: env::predecessor_account_id(),
            wormhole_core,
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

        let data: &[u8] = &vaa.payload;
        // TODO: parsing data to accountId, method, args
        // TODO: rewrite stub
        let recipient_account_id = "".parse().unwrap();
        let method = "".parse().unwrap();
        let args = Vec::new();

        let prepaid_gas = env::prepaid_gas().saturating_sub(VERIFY_CALL_GAS);
        let promise0 = env::promise_create(
            self.wormhole_core.clone(),
            "verify_vaa",
            &h,
            NearToken::from_near(0),
            prepaid_gas.saturating_sub(DELIVERY_CALL_GAS),
        );
        let promise1 = env::promise_then(
            promise0,
            recipient_account_id,
            &method,
            &args,
            NearToken::from_near(0),
            prepaid_gas.saturating_sub(DELIVERY_CALL_GAS),
        );
        let promise2 = env::promise_then(
            promise1,
            env::current_account_id(),
            "execute_delivery_callback",
            &Vec::new(),
            NearToken::from_near(0),
            CALL_CALL_GAS,
        );
        env::promise_return(promise2);
    }

    #[private]
    pub fn execute_delivery_callback(
        &self,
        #[callback_result] call_result: Result<(), PromiseError>,
    ) -> u64 {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
            env::panic_str("Delivery failed");
        }
        // TODO
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
            dups: UnorderedSet::new(b"d".to_vec()),
        }
    }
}


