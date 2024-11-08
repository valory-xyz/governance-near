use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseResult};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Call {
    pub contract_id: AccountId,
    pub method_name: String,
    pub args: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct CallResult {
    pub success: bool,
    pub result: Option<Vec<u8>>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MulticallContract {}

impl Default for MulticallContract {
    fn default() -> Self {
        Self {}
    }
}

#[near_bindgen]
impl MulticallContract {
    pub fn aggregate(&self, vaa: String, calls: Vec<Call>) -> Promise {
        // Сначала вызываем метод `verify` контракта wormhole.near
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicall_with_validation() {
        // TODO: tests
    }
}
