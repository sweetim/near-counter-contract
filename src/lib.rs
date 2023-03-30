use near_sdk::{near_bindgen};
use near_sdk::borsh::{self, BorshSerialize, BorshDeserialize};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub enum CounterAction {
    Increment,
    Decrement,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct CounterRecord {
    timestamp_ms: near_sdk::Timestamp,
    user: near_sdk::AccountId,
    action: CounterAction
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    value: u128,
    records: near_sdk::collections::Vector<CounterRecord>
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            value: 0,
            records: near_sdk::collections::Vector::new(b"r")
        }
    }
}

pub const STORAGE_FEE: u128 = 1_000_000_000_000_000_000_000;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn increment(&mut self) -> u128 {
        let user = near_sdk::env::signer_account_id();
        let fee = near_sdk::env::attached_deposit();
        let is_fee_sufficient = fee > STORAGE_FEE;

        near_sdk::require!(
            is_fee_sufficient,
            format!("insufficient near, please attach at least {STORAGE_FEE}"));

        self.value += 1;
        self.records.push(&CounterRecord {
            action: CounterAction::Increment,
            timestamp_ms: near_sdk::env::block_timestamp_ms(),
            user
        });

        self.value
    }

    #[payable]
    pub fn decrement(&mut self) -> u128 {
        let user = near_sdk::env::signer_account_id();
        let fee = near_sdk::env::attached_deposit();
        let is_fee_sufficient = fee > STORAGE_FEE;

        near_sdk::require!(
            is_fee_sufficient,
            format!("insufficient near, please attach at least {STORAGE_FEE}"));

        if self.value > 0 {
            self.value -= 1;
            self.records.push(&CounterRecord {
                action: CounterAction::Increment,
                timestamp_ms: near_sdk::env::block_timestamp_ms(),
                user
            });
        }

        self.value
    }

    pub fn get_value(&self) -> u128 {
        self.value
    }

    pub fn get_all_records(&self) -> Vec<CounterRecord> {
        self.records
            .iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_context(amount: Option<near_sdk::Balance>) {
        let context = near_sdk::test_utils::VMContextBuilder::new()
            .attached_deposit(amount.unwrap_or(1) * near_sdk::ONE_NEAR)
            .build();

        near_sdk::testing_env!(context)
    }

    #[test]
    fn get_count() {
        let contract = Contract::default();

        assert_eq!(0, contract.get_value());
    }

    #[test]
    fn increment() {
        set_context(None);

        let mut contract = Contract::default();
        assert_eq!(0, contract.get_value());

        contract.increment();
        assert_eq!(1, contract.get_value());

        contract.increment();
        assert_eq!(2, contract.get_value());

        contract.increment();
        assert_eq!(3, contract.get_value());
    }

    #[test]
    #[should_panic]
    fn increment_panic_when_insufficient() {
        set_context(Some(0));

        let mut contract = Contract::default();
        contract.increment();
    }

    #[test]
    fn decrement() {
        set_context(None);

        let mut contract = Contract::default();
        assert_eq!(0, contract.get_value());

        contract.increment();
        contract.increment();
        contract.increment();

        assert_eq!(3, contract.get_value());

        contract.decrement();
        assert_eq!(2, contract.get_value());

        contract.decrement();
        assert_eq!(1, contract.get_value());
    }

    #[test]
    #[should_panic]
    fn decrement_panic_when_insufficient() {
        set_context(Some(0));

        let mut contract = Contract::default();
        contract.decrement();
    }

    #[test]
    fn decrement_no_overflow() {
        set_context(None);

        let mut contract = Contract::default();
        assert_eq!(0, contract.get_value());

        contract.decrement();
        contract.decrement();
        contract.decrement();

        assert_eq!(0, contract.get_value());
    }

    #[test]
    fn get_all_records_default() {
        let contract = Contract::default();

        assert_eq!(0, contract.get_all_records().len());
    }
}
