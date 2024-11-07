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
    pub fn aggregate(&self, calls: Vec<Call>) {
        for call in calls.iter() {
            Promise::new(call.contract_id.clone()).function_call(
                call.method_name.clone(),
                call.args.clone(),
                0,      // attached deposit
                10_000_000_000_000,  // attached gas
            );
        }
    }

    pub fn on_aggregate_results(&self, calls: Vec<Call>) -> Vec<CallResult> {
        let mut results = Vec::new();

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

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multicall() {
        // Написать тесты для проверки контрактов
    }
}

