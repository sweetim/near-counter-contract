use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::serde::Serialize;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum CounterAction {
    Increment,
    Decrement,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct CounterRecord {
    timestamp_ms: near_sdk::Timestamp,
    user: near_sdk::AccountId,
    action: CounterAction,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    value: u128,
    records: near_sdk::collections::Vector<CounterRecord>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            value: 0,
            records: near_sdk::collections::Vector::new(b"r"),
        }
    }
}

pub const STORAGE_FEE: u128 = 1_000_000_000_000_000_000_000;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn increment(&mut self) -> U128 {
        U128(self.perform_action(CounterAction::Increment))
    }

    #[payable]
    pub fn decrement(&mut self) -> U128 {
        U128(self.perform_action(CounterAction::Decrement))
    }

    fn perform_action(&mut self, action: CounterAction) -> u128 {
        let user = near_sdk::env::signer_account_id();
        let fee = near_sdk::env::attached_deposit();
        let is_fee_sufficient = fee > STORAGE_FEE;

        near_sdk::require!(
            is_fee_sufficient,
            format!("insufficient near, please attach at least {STORAGE_FEE}")
        );

        self.value = Self::calculate_value(self.value, &action);

        self.records.push(&CounterRecord {
            action,
            timestamp_ms: near_sdk::env::block_timestamp_ms(),
            user,
        });

        self.value
    }

    fn calculate_value(input: u128, action: &CounterAction) -> u128 {
        match action {
            CounterAction::Increment => input + 1,
            CounterAction::Decrement => {
                if input > 0 {
                    input - 1
                } else {
                    0
                }
            }
        }
    }

    pub fn get_value(&self) -> U128 {
        U128(self.value)
    }

    pub fn query_all_records(&self) -> Vec<CounterRecord> {
        self.records.iter().collect()
    }

    pub fn query_records(
        &self,
        from_index: Option<U128>,
        limit: Option<U128>,
    ) -> Vec<CounterRecord> {
        self.records
            .iter()
            .skip(from_index.unwrap_or(U128(0)).0 as usize)
            .take(limit.unwrap_or(U128(10)).0 as usize)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_context(account_id: Option<&str>, amount: Option<near_sdk::Balance>) {
        let context = near_sdk::test_utils::VMContextBuilder::new()
            .attached_deposit(amount.unwrap_or(1) * near_sdk::ONE_NEAR)
            .signer_account_id(account_id.unwrap_or("default.test").parse().unwrap())
            .build();

        near_sdk::testing_env!(context)
    }

    #[test]
    fn get_count() {
        let contract = Contract::default();

        assert_eq!(U128(0), contract.get_value());
    }

    #[test]
    fn calculate_value_increment() {
        assert_eq!(11, Contract::calculate_value(10, &CounterAction::Increment));
    }

    #[test]
    fn calculate_value_decrement() {
        assert_eq!(9, Contract::calculate_value(10, &CounterAction::Decrement));
    }

    #[test]
    fn calculate_value_decrement_no_overflow() {
        assert_eq!(0, Contract::calculate_value(0, &CounterAction::Decrement));
        assert_eq!(0, Contract::calculate_value(0, &CounterAction::Decrement));
    }

    #[test]
    fn increment() {
        set_context(None, None);

        let mut contract = Contract::default();
        assert_eq!(U128(0), contract.get_value());

        contract.increment();
        assert_eq!(U128(1), contract.get_value());

        contract.increment();
        assert_eq!(U128(2), contract.get_value());

        contract.increment();
        assert_eq!(U128(3), contract.get_value());
    }

    #[test]
    #[should_panic]
    fn increment_panic_when_insufficient() {
        set_context(None, Some(0));

        let mut contract = Contract::default();
        contract.increment();
    }

    #[test]
    fn decrement() {
        set_context(None, None);

        let mut contract = Contract::default();
        assert_eq!(U128(0), contract.get_value());

        contract.increment();
        contract.increment();
        contract.increment();

        assert_eq!(U128(3), contract.get_value());

        contract.decrement();
        assert_eq!(U128(2), contract.get_value());

        contract.decrement();
        assert_eq!(U128(1), contract.get_value());
    }

    #[test]
    #[should_panic]
    fn decrement_panic_when_insufficient() {
        set_context(None, Some(0));

        let mut contract = Contract::default();
        contract.decrement();
    }

    #[test]
    fn decrement_no_overflow() {
        set_context(None, None);

        let mut contract = Contract::default();
        assert_eq!(U128(0), contract.get_value());

        contract.decrement();
        contract.decrement();
        contract.decrement();

        assert_eq!(U128(0), contract.get_value());
    }

    #[test]
    fn query_all_records_default() {
        let contract = Contract::default();

        assert_eq!(0, contract.query_all_records().len());
    }

    #[test]
    fn query_all_records() {
        let mut contract = Contract::default();

        set_context(Some("user_1"), None);
        contract.increment();
        assert_eq!(1, contract.query_all_records().len());

        set_context(Some("user_2"), None);
        contract.increment();
        assert_eq!(2, contract.query_all_records().len());

        set_context(Some("user_1"), None);
        contract.decrement();
        assert_eq!(3, contract.query_all_records().len());

        assert_eq!(
            vec![
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: "user_1".parse().unwrap()
                },
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: "user_2".parse().unwrap()
                },
                CounterRecord {
                    action: CounterAction::Decrement,
                    timestamp_ms: 0,
                    user: "user_1".parse().unwrap()
                }
            ],
            contract.query_all_records()
        );
    }
}
