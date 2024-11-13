use near_sdk::borsh::{self, BorshDeserialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::collections::UnorderedSet;
use near_sdk::{
    env, near, ext_contract, require, AccountId, Promise, PromiseOrValue, Gas, PromiseResult, NearToken, log, serde_json
};

pub mod byte_utils;
pub mod state;

// Wormhole Core interface
#[ext_contract(wormhole)]
pub trait Wormhole {
    fn verify_vaa(&self, vaa: String) -> u32;
}

// Prepaid gas for a single (not inclusive of recursion) `verify_vaa` call.
const VERIFY_CALL_GAS: Gas = Gas::from_tgas(20);
const DELIVERY_CALL_GAS: Gas = Gas::from_tgas(100);
const CALL_CALL_GAS: Gas = Gas::from_tgas(5);
const MAX_NUM_CALLS: usize = 10;

#[derive(BorshDeserialize, Serialize, Deserialize)]
pub struct Call {
    pub contract_id: AccountId,
    pub deposit: u64,
    pub method_name: String,
    pub args: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CallResult {
    pub success: bool,
    pub result: Option<Vec<u8>>,
}

#[near(contract_state)]
pub struct WormholeRelayer {
    owner: AccountId,
    wormhole_core: AccountId,
    foreign_governor_address: Vec<u8>,
    chain_id: u16,
    dups: UnorderedSet<Vec<u8>>,
}

impl Default for WormholeRelayer {
    fn default() -> Self {
        Self {
            owner: "".parse().unwrap(),
            wormhole_core: "".parse().unwrap(),
            foreign_governor_address: Vec::new(),
            chain_id: 0,
            dups: UnorderedSet::new(b"d".to_vec()),
        }
    }
}

#[near]
impl WormholeRelayer {
    #[init]
    pub fn new(owner_id: AccountId, wormhole_core: AccountId, foreign_governor_address: Vec<u8>, chain_id: u16) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: owner_id,
            wormhole_core,
            foreign_governor_address,
            chain_id,
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

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }

    pub fn to_bytes(&self, calls: Vec<Call>) -> Vec<u8> {
        serde_json::to_vec(&calls).expect("Failed to serialize Vec<Call>")
    }

    #[payable]
    pub fn delivery(&mut self, vaa: String) -> Promise {
        let h = hex::decode(vaa.clone()).expect("invalidVaa");
        let parsed_vaa = state::ParsedVAA::parse(&h);

        if self.dups.contains(&parsed_vaa.hash) {
            env::panic_str("alreadyExecuted");
        }

        log!("parsed_vaa: {:?}", parsed_vaa);

        let initial_storage_usage = env::storage_usage();
        // TODO enable in production
        //self.dups.insert(&parsed_vaa.hash);
        let storage = env::storage_usage() - initial_storage_usage;

        if parsed_vaa.emitter_chain != self.chain_id || parsed_vaa.emitter_address != self.foreign_governor_address {
            env::panic_str("InvalidGovernorEmitter");
        }

        let data: &[u8] = &parsed_vaa.payload;
        log!("data: {:?}", data);
        let calls: Vec<Call> = serde_json::from_slice(data).expect("Failed to deserialize Vec<Call>");

        // TODO Set a limit for calls.len
        require!(calls.len() <= MAX_NUM_CALLS, "Exceeded max number of calls");

        let attached_deposit = env::attached_deposit();
        let mut sum_deposit = 0 as u64;
        for call in calls.iter() {
            sum_deposit += call.deposit;

            log!("call contract_id: {}", call.contract_id);
            log!("call deposit: {}", call.deposit);
            log!("call method_name: {}", call.method_name);
            log!("call args: {:?}", call.args);
        }
        let refund = attached_deposit.saturating_sub(NearToken::from_yoctonear(sum_deposit.into()));

        self.refund_deposit_to_account(storage, refund, env::predecessor_account_id(), true);

        let promise = Promise::new(self.wormhole_core.clone())
            .function_call(
                "verify_vaa".to_string(),
                serde_json::json!({ "vaa": vaa }).to_string().as_bytes().to_vec(),
                NearToken::from_yoctonear(0),
                VERIFY_CALL_GAS,
            );

        // Pass all the calls and 0-th index of a promise
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(DELIVERY_CALL_GAS)
                .with_attached_deposit(attached_deposit)
                .on_complete(calls, 0),
        )
    }

    #[private]
    pub fn on_complete(&self, calls: Vec<Call>, index: usize) -> PromiseOrValue<CallResult> {
        // Check the VAA verification
        if let PromiseResult::Successful(_) = env::promise_result(0) {
            if index < calls.len() {
                let call = &calls[index];
                let next_promise = Promise::new(call.contract_id.clone())
                    .function_call(
                        call.method_name.clone(),
                        call.args.clone(),
                        NearToken::from_yoctonear(call.deposit.clone().into()),
                        CALL_CALL_GAS,
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(DELIVERY_CALL_GAS)
                            .on_complete(calls, index + 1)
                    );
                PromiseOrValue::Promise(next_promise)
            } else {
                // No more calls in stack, return success
                PromiseOrValue::Value(CallResult { success: true, result: Some("Ok".into()) })
            }
        } else {
            // Return fail
            PromiseOrValue::Value(CallResult { success: false, result: None })
        }
    }

    fn refund_deposit_to_account(&self, storage_used: u64, service_deposit: NearToken, account_id: AccountId, deposit_in: bool) {
        let mut required_cost = env::storage_byte_cost().saturating_mul(storage_used.into());
        required_cost = required_cost.saturating_add(service_deposit);

        let mut refund = env::attached_deposit().into();
        if deposit_in {
            require!(required_cost <= refund);
            refund = refund.saturating_sub(required_cost);
        } else {
            require!(required_cost <= env::account_balance());
            refund = refund.saturating_add(required_cost);
        }
        if refund > NearToken::from_yoctonear(1) {
            Promise::new(account_id).transfer(refund);
        }
    }
}

