use near_sdk::near_bindgen;
use near_sdk::json_types::U128;
use near_sdk::serde::Serialize;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

mod events;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum CounterAction {
    Increment,
    Decrement,
    Random,
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

pub const COUNTER_ENTRY_FEE: u128 = 10_000_000_000_000_000_000_000;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn increment(&mut self) -> U128 {
        self.perform_action(CounterAction::Increment)
    }

    #[payable]
    pub fn decrement(&mut self) -> U128 {
        self.perform_action(CounterAction::Decrement)
    }

    #[payable]
    pub fn random(&mut self) -> U128 {
        self.perform_action(CounterAction::Random)
    }

    fn perform_action(&mut self, action: CounterAction) -> U128 {
        let user = near_sdk::env::signer_account_id();
        let fee = near_sdk::env::attached_deposit();
        let is_fee_sufficient = fee >= COUNTER_ENTRY_FEE;

        near_sdk::require!(
            is_fee_sufficient,
            format!("insufficient near, please attach at least {COUNTER_ENTRY_FEE}")
        );

        self.value = Self::calculate_value(self.value, &action);
        near_sdk::log!(events::CounterEventLog::create(&action, self.value).to_string());

        self.records.push(&CounterRecord {
            action,
            timestamp_ms: near_sdk::env::block_timestamp_ms(),
            user,
        });

        U128(self.value)
    }

    fn calculate_value(input: u128, action: &CounterAction) -> u128 {
        match action {
            CounterAction::Increment => input.checked_add(1).unwrap_or(input),
            CounterAction::Decrement => input.checked_sub(1).unwrap_or(0),
            CounterAction::Random => {
                let random_action = near_sdk::env::random_seed()
                    .into_iter()
                    .reduce(|acc, item| acc ^ item)
                    .map(|item| item % 2 == 0)
                    .map(|item| match item {
                        true => CounterAction::Increment,
                        false => CounterAction::Decrement
                    })
                    .unwrap();

                Self::calculate_value(input, &random_action)
            }
        }
    }

    pub fn get_entry_fee(&self) -> U128 {
        U128(COUNTER_ENTRY_FEE)
    }

    pub fn get_value(&self) -> U128 {
        U128(self.value)
    }

    pub fn get_records_length(&self) -> u64 {
        self.records.len()
    }

    pub fn query_all_records(&self) -> Vec<CounterRecord> {
        self.query_records(None, None)
    }

    pub fn query_records(
        &self,
        from_index: Option<U128>,
        limit: Option<U128>,
    ) -> Vec<CounterRecord> {
        self.records
            .iter()
            .rev()
            .skip(from_index.unwrap_or(U128(0)).0 as usize)
            .take(limit.unwrap_or(U128(u128::MAX)).0 as usize)
            .collect()
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    fn set_context(account_id: Option<near_sdk::AccountId>, amount: Option<near_sdk::Balance>) {
        let context = near_sdk::test_utils::VMContextBuilder::new()
            .attached_deposit(amount.unwrap_or(1) * near_sdk::ONE_NEAR)
            .signer_account_id(account_id.unwrap_or(near_sdk::test_utils::accounts(0)))
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
    fn calculate_value_random() {
        assert_eq!(11, Contract::calculate_value(10, &CounterAction::Random));
        assert_eq!(12, Contract::calculate_value(11, &CounterAction::Random));
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
    fn increment_no_overflow() {
        set_context(None, None);

        let mut contract = Contract::default();
        contract.value = u128::MAX;

        assert_eq!(U128(u128::MAX), contract.get_value());

        contract.increment();
        contract.increment();
        contract.increment();

        assert_eq!(U128(u128::MAX), contract.get_value());
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

        assert_eq!(0, contract.get_records_length());
        assert_eq!(0, contract.query_all_records().len());
    }

    #[test]
    fn query_all_records() {
        let mut contract = Contract::default();

        set_context(Some(near_sdk::test_utils::accounts(1)), None);
        contract.increment();
        assert_eq!(1, contract.query_all_records().len());

        set_context(Some(near_sdk::test_utils::accounts(2)), None);
        contract.increment();
        assert_eq!(2, contract.query_all_records().len());

        set_context(Some(near_sdk::test_utils::accounts(3)), None);
        contract.decrement();
        assert_eq!(3, contract.query_all_records().len());

        assert_eq!(3, contract.get_records_length());
        assert_eq!(
            contract.query_all_records(),
            vec![
                CounterRecord {
                    action: CounterAction::Decrement,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(3)
                },
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(2)
                },
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(1)
                },
            ]
        );
    }

    #[test]
    fn query_records() {
        let mut contract = Contract::default();

        set_context(Some(near_sdk::test_utils::accounts(1)), None);
        contract.increment();
        set_context(Some(near_sdk::test_utils::accounts(2)), None);
        contract.increment();
        set_context(Some(near_sdk::test_utils::accounts(3)), None);
        contract.decrement();

        assert_eq!(
            contract.query_records(Some(U128(1)), None),
            vec![
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(2)
                },
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(1)
                },
            ],
        );

        assert_eq!(
            contract.query_records(Some(U128(1)), Some(U128(1))),
            vec![
                CounterRecord {
                    action: CounterAction::Increment,
                    timestamp_ms: 0,
                    user: near_sdk::test_utils::accounts(2)
                }
            ],
        );
    }
}
