use near_sdk::borsh::{self, BorshDeserialize};
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::store::iterable_set::IterableSet;
use near_sdk::{
    env, near, ext_contract, require, AccountId, Promise, PromiseOrValue, Gas, PromiseResult, NearToken, serde_json
};

pub mod byte_utils;
pub mod state;

// Wormhole Core interface
#[ext_contract(wormhole)]
pub trait Wormhole {
    fn verify_vaa(&self, vaa: String) -> u32;
}

// Prepaid gas for a single (not inclusive of recursion) `verify_vaa` call.
const COMPLETE_CALL_GAS_NUM: u64 = 5;
const VERIFY_CALL_GAS: Gas = Gas::from_tgas(VERIFY_CALL_GAS_NUM);
// Prepaid gas for VAA verification
const VERIFY_CALL_GAS_NUM: u64 = 20;
const COMPLETE_CALL_GAS: Gas = Gas::from_tgas(COMPLETE_CALL_GAS_NUM);
// Maximum number of calls
const MAX_NUM_CALLS: usize = 10;

// Call struct: target, value, gas, selector, arguments
#[derive(BorshDeserialize, Serialize, Deserialize, Debug)]
pub struct Call {
    // Target contract
    pub contract_id: AccountId,
    // Deposit value amount
    pub deposit: NearToken,
    // Function call gas
    pub gas: u64,
    // Selector name
    pub method_name: String,
    // Arguments as vector of bytes
    pub args: Vec<u8>,
}

// Call result: success and result as vector of bytes
#[derive(Serialize, Deserialize, Debug)]
pub struct CallResult {
    pub success: bool,
    pub result: Option<Vec<u8>>,
}

// WormholeMessenger contract:
#[near(contract_state)]
pub struct WormholeMessenger {
    // Wormhole Core account Id
    wormhole_core: AccountId,
    // Foreign governor emitter in bytes form
    foreign_governor_emitter: Vec<u8>,
    // Foreign chain Id
    foreign_chain_id: u16,
    // Processed VAA hashes
    dups: IterableSet<Vec<u8>>,
    // Contract upgrade hash
    upgrade_hash: Vec<u8>,
}

impl Default for WormholeMessenger {
    fn default() -> Self {
        Self {
            wormhole_core: "".parse().unwrap(),
            foreign_governor_emitter: Vec::new(),
            foreign_chain_id: 0,
            dups: IterableSet::new(b"d".to_vec()),
            upgrade_hash: Vec::new()
        }
    }
}

#[near]
impl WormholeMessenger {
    #[init]
    pub fn new(
        wormhole_core: AccountId,
        foreign_governor_emitter: Vec<u8>,
        foreign_chain_id: u16
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            wormhole_core,
            foreign_governor_emitter,
            foreign_chain_id,
            dups: IterableSet::new(b"d".to_vec()),
            upgrade_hash: b"h".to_vec()
        }
    }

    #[private]
    pub fn refund_deposit_to_account(&self, storage_used: u64, deposit: NearToken, account_id: AccountId) {
        let mut required_cost = env::storage_byte_cost().saturating_mul(storage_used.into());
        required_cost = required_cost.saturating_add(deposit);

        let mut refund = env::attached_deposit().into();
        require!(required_cost <= refund, "Insufficient required cost");
        refund = refund.saturating_sub(required_cost);

        if refund > NearToken::from_yoctonear(1) {
            Promise::new(account_id).transfer(refund);
        }
    }

    #[private]
    pub fn change_foreign_governor_address(&mut self, new_foreign_governor_address: Vec<u8>) {
        // Check account validity
        require!(env::is_valid_account_id(&new_foreign_governor_address), "Account Id is invalid");

        self.foreign_governor_emitter = new_foreign_governor_address.clone();

        env::log_str(&format!(
            "WormholeMessenger/{}#{}: : {}",
            file!(),
            line!(),
            hex::encode(&new_foreign_governor_address)
        ));
    }

    #[private]
    pub fn change_upgrade_hash(&mut self, hash: Vec<u8>) {
        env::log_str(&format!(
            "WormholeMessenger/{}#{}: update_contract_hash: {}",
            file!(),
            line!(),
            hex::encode(&hash)
        ));

        self.upgrade_hash = hash;
    }

    #[private]
    pub fn process_vaa(&mut self, vaa: String) -> Vec<Call> {
        let h = hex::decode(vaa.clone()).expect("invalidVaa");
        let parsed_vaa = state::ParsedVAA::parse(&h);

        if self.dups.contains(&parsed_vaa.hash) {
            env::panic_str("AlreadyExecuted");
        }

        // Record processed vaa
        self.dups.insert(parsed_vaa.hash);
        self.dups.flush();

        if parsed_vaa.emitter_chain != self.foreign_chain_id || parsed_vaa.emitter_address != self.foreign_governor_emitter {
            env::panic_str("InvalidGovernorEmitter");
        }

        let data = &parsed_vaa.payload;
        let calls: Vec<Call> = serde_json::from_slice(data).expect("Failed to deserialize Vec<Call>");
        calls
    }

    #[private]
    pub fn on_complete(&self, calls: Vec<Call>, index: usize) -> PromiseOrValue<CallResult> {
        // Check the VAA verification
        if let PromiseResult::Successful(_) = env::promise_result(0) {
            if index < calls.len() {
                let call = &calls[index];

                env::log_str(&format!(
                    "WormholeMessenger/{}#{}: : {:?}",
                    file!(),
                    line!(),
                    call
                ));

                let next_promise = Promise::new(call.contract_id.clone())
                    .function_call(
                        call.method_name.clone(),
                        call.args.clone(),
                        call.deposit.clone(),
                        Gas::from_tgas(call.gas.clone()),
                    )
                    .then(
                        Self::ext(env::current_account_id())
                            .with_static_gas(COMPLETE_CALL_GAS)
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

    #[payable]
    pub fn delivery(&mut self, vaa: String) -> Promise {
        let initial_storage_usage = env::storage_usage();

        let calls = self.process_vaa(vaa.clone());

        let storage = env::storage_usage() - initial_storage_usage;

        require!(calls.len() <= MAX_NUM_CALLS, "Exceeded max number of calls");

        let mut sum_deposit = NearToken::from_yoctonear(0);
        let mut sum_gas = 0;
        for call in calls.iter() {
            sum_deposit = sum_deposit.saturating_add(call.deposit);
            sum_gas = sum_gas + call.gas + COMPLETE_CALL_GAS_NUM;
        }
        sum_gas = sum_gas + VERIFY_CALL_GAS_NUM + COMPLETE_CALL_GAS_NUM;
        require!(env::prepaid_gas() > Gas::from_tgas(sum_gas), "Exceeded max gas");

        // Refund sender account
        self.refund_deposit_to_account(storage, sum_deposit, env::predecessor_account_id());

        let promise = Promise::new(self.wormhole_core.clone())
            .function_call(
                "verify_vaa".to_string(),
                serde_json::json!({ "vaa": vaa }).to_string().as_bytes().to_vec(),
                NearToken::from_yoctonear(0),
                VERIFY_CALL_GAS
            );

        // Pass all the calls and 0-th index of a promise
        promise.then(
            Self::ext(env::current_account_id())
                .with_static_gas(Gas::from_tgas(sum_gas))
                .with_attached_deposit(sum_deposit)
                .on_complete(calls, 0),
        )
    }

	pub fn upgrade_contract(&self) {
        // Receive the code directly from the input to avoid the
        // GAS overhead of deserializing parameters
        let code = env::input().expect("Error: No input").to_vec();

        let hash = env::sha256(&code);

        // Check if caller is authorized to update the contract code
        if hash != self.upgrade_hash {
           env::panic_str("InvalidUpgradeContractHash");
        }

        env::log_str(&format!(
            "WormholeMessenger/{}#{}: : {}",
            file!(),
            line!(),
            hex::encode(&hash)
        ));

        // Deploy the contract on self
        Promise::new(env::current_account_id())
            .deploy_contract(code);
    }

    pub fn to_bytes(&self, calls: Vec<Call>) -> Vec<u8> {
        serde_json::to_vec(&calls).expect("Failed to serialize Vec<Call>")
    }

    pub fn get_foreign_governor_emitter(&self) -> Vec<u8> {
        self.foreign_governor_emitter.clone()
    }

    pub fn get_foreign_chain_id(&self) -> u16 {
        self.foreign_chain_id
    }

    pub fn get_storage_usage(&self) -> u64 {
        env::storage_usage()
    }

    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}
