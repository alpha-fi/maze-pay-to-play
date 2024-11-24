use near_contract_standards::fungible_token::Balance;
// Find all our documentation at https://docs.near.org
use near_sdk::{
    env, log, near_bindgen, require, AccountId, PanicOnDefault
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::collections::HashMap;
use utils::to_yocto_u8;

mod internal;
mod deposit;
mod utils;

pub type Day = u64;
pub type GameAmount = u16;

const DAY_MS: u64 = 24 * 3600 * 1000;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct FreeGameInfo {
	day: Day,
	amount: GameAmount,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MazeGameBuyerContract {
    owner_id: AccountId,
    cheddar_contract: AccountId,
    game_costs: HashMap<u8, Balance>,
    user_remaining_free_games: HashMap<AccountId, FreeGameInfo>,
    user_remaining_paid_games: HashMap<AccountId, GameAmount>,

}


// Implement the contract structure
#[near_bindgen]
impl MazeGameBuyerContract {

    #[init]
    pub fn new(cheddar_contract: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let owner_id = env::predecessor_account_id();
        let mut game_costs: HashMap<u8, u128> = HashMap::new();
        
        game_costs.insert(1, to_yocto_u8(15));
        Self {
            owner_id,
            cheddar_contract,
            game_costs,
            user_remaining_free_games: HashMap::new(),
            user_remaining_paid_games: HashMap::new(),
        }
    }

    pub fn get_games_costs(&self) -> HashMap<u8, Balance> {
        self.game_costs.clone()
    }

    // Ensure game_costs always has 1 as key, and at most 4 keys
    pub fn set_game_costs(&mut self, game_costs: HashMap<u8, Balance>) {
        self.assert_only_owner();
        log!("Saving game costs: {:?}", game_costs);
        self.game_costs = game_costs;
    }

    pub fn get_cheddar_contract(&self) -> String {
        self.cheddar_contract.to_string()
    }

    pub fn set_cheddar_contract(&mut self, cheddar_contract: AccountId) {
        self.assert_only_owner();
        self.cheddar_contract = cheddar_contract;
    }

    pub fn get_user_remaining_free_games(&self, account_id: AccountId) -> GameAmount {
        let day = env::block_timestamp_ms() / DAY_MS;
        log!("Day: {}", day);
        let user_free_remaining_games_data = self.user_remaining_free_games.get(&account_id).unwrap_or(&FreeGameInfo {
            day: 0,
            amount: 0,
        });
        if day == user_free_remaining_games_data.day {
            user_free_remaining_games_data.amount
        } else {
            5
        }
    }

    pub fn give_free_game_to_user(&mut self, account_id: AccountId) {
        self.assert_only_owner();
        let day = env::block_timestamp_ms() / DAY_MS;

        let user_remaining_free_games = self.get_user_remaining_free_games(account_id.clone());

        self.user_remaining_free_games.insert(account_id.clone(), FreeGameInfo {
            day,
            amount: user_remaining_free_games + 1,
        });
    }

    pub fn get_user_remaining_paid_games(&self, account_id: AccountId) -> GameAmount {
        *self.user_remaining_paid_games.get(&account_id).unwrap_or(&0)
    }

    fn add_games_to_user(&mut self, account_id: AccountId, amount: GameAmount) {
        let user_remaining_paid_games = self.get_user_remaining_paid_games(account_id.clone());
        self.user_remaining_paid_games.insert(account_id, user_remaining_paid_games + amount);
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use near_sdk::{test_utils::{accounts, VMContextBuilder}, testing_env};

    use super::*;

    const MSECOND: u64 = 1_000_000;

    fn setup_contract() -> (VMContextBuilder, MazeGameBuyerContract) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).block_timestamp(DAY_MS * MSECOND).build());
        let cheddar_contract = AccountId::from_str("token.cheddar.near").unwrap();
        let contract = MazeGameBuyerContract::new(cheddar_contract);
        (context, contract)
    }

    #[test]
    fn get_default_game_costs() {
        let (_, contract) = setup_contract();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_games_costs(), vec![(1, to_yocto_u8(15))].into_iter().collect());
    }

    #[test]
    fn set_then_get_game_costs() {
        let (_, mut contract) = setup_contract();
        contract.set_game_costs(vec![(1, 20)].into_iter().collect());
        assert_eq!(contract.get_games_costs(), vec![(1, 20)].into_iter().collect());
    }

    #[test]
    fn get_default_cheddar_contract() {
        let (_, contract) = setup_contract();
        let cheddar_contract = AccountId::from_str("token.cheddar.near").unwrap();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_cheddar_contract(), cheddar_contract);
    }

    #[test]
    fn set_then_get_cheddar_contract() {
        let (_, mut contract) = setup_contract();
        let cheddar_contract: AccountId = AccountId::from_str("token-v2.cheddar.near").unwrap();
        contract.set_cheddar_contract(cheddar_contract.clone());
        assert_eq!(contract.get_cheddar_contract(), cheddar_contract);
    }

    #[test]
    fn get_default_free_games() {
        let (_, contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(contract.get_user_remaining_free_games(user), 5);
    }

    #[test]
    fn set_then_get_free_games() {
        let (_, mut contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        contract.give_free_game_to_user(user.clone());
        assert_eq!(contract.get_user_remaining_free_games(user), 6);
    }

    #[test]
    fn get_free_games_on_new_day() {
        let (mut context, mut contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        contract.give_free_game_to_user(user.clone());
        testing_env!(context.block_timestamp(2 * DAY_MS * MSECOND).build());
        assert_eq!(contract.get_user_remaining_free_games(user), 5);
    }

}
