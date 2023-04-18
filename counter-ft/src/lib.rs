use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const COUNTER_SVG_ICON: &str = "data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 258 205' width='64px' height='64px' style='fill:white'%3e%3cpath d='M149.3 35.5c-1.7.7-3.6 2.2-4.2 3.4-.7 1.2-1.1 10.2-1.1 24.3v22.3l-12.6 6.3c-7 3.5-12.7 6.2-12.9 6-.1-.2-.7-12.2-1.3-26.8-1-23.2-1.4-26.8-3-29-2.3-3.1-8.2-4.7-12-3.4-4.9 1.7-5.4 4.4-4.3 23.1.6 9.2 1.1 23.5 1.2 31.8l.1 15-4.4 2.6c-7.1 4.2-24.9 13.2-25.3 12.8-.4-.5 1.1-53.6 2.1-71.1l.7-12.6-2.6-2.6c-3.5-3.5-9.4-3.6-13.3-.3-2.5 2.2-2.8 3.2-3.5 12.4-.4 5.5-1 20.8-1.3 33.9-.4 13.1-.9 30.3-1.2 38l-.6 14.1-12.1 6.9c-6.7 3.8-13.2 7.9-14.4 9.1-3.3 3.2-3.1 8.4.6 12.5 4.1 4.6 7.7 4.3 17.6-1.5l8-4.6.6 11.2c.6 12.8 1.1 14.2 5 16.2s8.2 1.9 11-.4c2.3-1.9 2.4-2.5 2.9-20.2l.5-18.4 14.5-8.2c8-4.5 14.8-8.2 15.3-8.2.4-.1.7 11.1.7 24.8 0 13.7.3 25.7.6 26.6.4.8 2 2.3 3.6 3.1 3.9 2 8.8 1.1 11.9-2.2l2.4-2.6.3-30.2.3-30.2 12.5-6.2c6.8-3.4 12.6-6.2 12.9-6.2.3 0 .5 16.4.5 36.4v36.5l2.4 2.8c3.2 3.7 10.3 4 13.8.5l2.3-2.3.5-42 .5-42.1 10.4-4.9 10.4-4.9-.8 43.8c-.4 24-.7 45.4-.8 47.5-.1 5.3 2.9 8.2 8.8 8.2 3.4 0 5-.6 7-2.6 2.5-2.4 2.6-3.2 3.5-19 .5-9 .9-32.2 1-51.4V78.4l2.8-1.1c18.1-7.3 22.1-9.5 23.7-13 1.8-3.7 1.4-7.1-1-10.9-2.5-3.9-5.8-3.8-15.7.6-5 2.2-9.4 4-9.8 4-.4 0-1-3.9-1.3-8.6-.7-9.5-2.2-12.9-6.5-14.5-3.6-1.4-8.5-.4-10.8 2.2-1.7 1.9-1.9 3.8-1.9 16.2v14l-9.5 4.3c-5.2 2.4-9.8 4.3-10.2 4.3-.3.1-.8-8-1-17.9-.3-17.7-.3-18-2.8-20.4-3.3-3.1-6.7-3.8-10.7-2.1z'/%3e%3c/svg%3e";

const MAX_TOKEN_SUPPLY: u128 = 1_000_000;
const TOTAL_TOKEN_RECEIVE_FROM_EACH_ACTION: u128 = 1;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    FungibleToken,
    Metadata,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        require!(!env::state_exists(), "already initialized");

        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "COUNTER".to_string(),
            symbol: "CNTR".to_string(),
            icon: Some(COUNTER_SVG_ICON.to_string()),
            reference: None,
            reference_hash: None,
            decimals: 0,
        };

        metadata.assert_valid();

        Self {
            token: FungibleToken::new(StorageKey::FungibleToken),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
        }
    }

    pub fn ft_mint(&mut self) {
        // let is_from_counter_contract = true;

        let signer_account_id = near_sdk::env::signer_account_id();

        let new_total_supply = self.token.total_supply.checked_add(TOTAL_TOKEN_RECEIVE_FROM_EACH_ACTION)
            .unwrap_or_else(|| near_sdk::env::panic_str("total supply overflow"));

        if new_total_supply <= MAX_TOKEN_SUPPLY {
            self.token.internal_deposit(&signer_account_id, TOTAL_TOKEN_RECEIVE_FROM_EACH_ACTION);

            near_contract_standards::fungible_token::events::FtMint {
                owner_id: &signer_account_id,
                amount: &U128(TOTAL_TOKEN_RECEIVE_FROM_EACH_ACTION),
                memo: None,
            }.emit();
        } else {
            near_sdk::env::panic_str("all tokens minted");
        }
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_context(account_id: AccountId, deposit: u128) {
        let context = near_sdk::test_utils::VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id)
            .attached_deposit(deposit * near_sdk::ONE_NEAR)
            .build();

        near_sdk::testing_env!(context)
    }

    #[test]
    fn new() {
        let contract = Contract::new();

        assert_eq!(contract.ft_total_supply().0, 0);
        assert_eq!(contract.ft_balance_of(near_sdk::test_utils::accounts(1)).0, 0);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn default() {
        Contract::default();
    }

    #[test]
    fn ft_mint() {
        let account_id = near_sdk::test_utils::accounts(0);
        get_context(account_id.clone(), 1);

        let mut contract = Contract::new();
        contract.storage_deposit(None, None);
        contract.ft_mint();

        assert_eq!(U128(1), contract.ft_balance_of(account_id));
    }
}
