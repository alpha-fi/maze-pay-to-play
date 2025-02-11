use external::ext_maze_minter;
use near_contract_standards::fungible_token::Balance;
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::store::IterableMap;
// Find all our documentation at https://docs.near.org
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Gas, NearToken, PanicOnDefault, Promise
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use schemars::JsonSchema;
use utils::{get_now_ms, get_today_day, to_yocto_u8};
use structs::game_struct_json::GameJson;

mod internal;
mod deposit;
mod utils;
mod structs;
mod external;

pub type Day = u64; // Having this data type, means how many days passed since 01/01/1970
pub type GameAmount = u16;
pub type SeedId = u64;

const DAY_MS: u64 = 24 * 3600 * 1000;
const MIN_MS: u64 = 60 * 1000;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct FreeGameInfo {
	day: Day,
	amount: GameAmount,
}
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, JsonSchema)]
pub struct Game {
	seed_id: SeedId,
	start_time: u64,
    is_ending_game: bool
}

impl Default for Game {
    fn default() -> Self {
        Self {
            seed_id: 0,
            start_time: 0,
            is_ending_game: false
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct MazeGameBuyerContract {
    owner_id: AccountId,
    cheddar_contract: AccountId,
    game_costs: IterableMap<u8, Balance>,
    user_remaining_free_games: UnorderedMap<AccountId, FreeGameInfo>,
    user_remaining_paid_games: UnorderedMap<AccountId, GameAmount>,
    seed_id: SeedId,
    min_deposit: Balance,
    ongoing_games: UnorderedMap<AccountId, Game>,
    maze_minter_contract: AccountId,
    max_game_duration: u64,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldMazeGameBuyerContract {
    owner_id: AccountId,
    cheddar_contract: AccountId,
    game_costs: IterableMap<u8, Balance>,
    user_remaining_free_games: UnorderedMap<AccountId, FreeGameInfo>,
    user_remaining_paid_games: UnorderedMap<AccountId, GameAmount>,
    seed_id: SeedId,
    min_deposit: Balance,
    ongoing_games: UnorderedMap<AccountId, Game>,
    maze_minter_contract: AccountId,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct ContractState {
	owner_id: String,
    cheddar_contract: String,
    maze_minter_contract: String,
    game_costs: Vec<[String; 2]>,
    seed_id: SeedId,
    min_deposit: String,
    max_game_duration: u64,
}


// Implement the contract structure
#[near_bindgen]
impl MazeGameBuyerContract {

    #[init]
    pub fn new(
        cheddar_contract: AccountId, 
        maze_minter_contract: AccountId
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        let owner_id = env::predecessor_account_id();
        let mut game_costs: IterableMap<u8, u128> = IterableMap::new(b"game_costs".to_vec());
        
        game_costs.insert(1, to_yocto_u8(15).0);
        game_costs.insert(10, to_yocto_u8(14).0);
        Self {
            owner_id,
            cheddar_contract,
            game_costs,
            user_remaining_free_games: UnorderedMap::new(b"free_games".to_vec()),
            user_remaining_paid_games: UnorderedMap::new(b"paid_games".to_vec()),
            seed_id: 0u64,
            min_deposit: 1_000_000_000_000_000_000_000, // 0.001 NEAR
            ongoing_games: UnorderedMap::new(b"ongoing_games".to_vec()),
            maze_minter_contract,
            max_game_duration: 3 * MIN_MS,
        }
    }

    // This hasn't been properly tested, but it is supposed to be the right way. Be sure to add the new variables in case of migrating
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        // Load the existing state
        let old_state: OldMazeGameBuyerContract = env::state_read().expect("Failed to read state");

        let mut new_user_remaining_free_games = UnorderedMap::new(b"g"); // Use a new storage prefix
        for (key, value) in old_state.user_remaining_free_games.iter() {
            new_user_remaining_free_games.insert(&key, &value);
        }

        let mut new_user_remaining_paid_games = UnorderedMap::new(b"g"); // Use a new storage prefix
        for (key, value) in old_state.user_remaining_paid_games.iter() {
            new_user_remaining_paid_games.insert(&key, &value);
        }

        let mut new_ongoing_games = UnorderedMap::new(b"g"); // Use a new storage prefix
        for (key, value) in old_state.ongoing_games.iter() {
            new_ongoing_games.insert(&key, &value);
        }
        // Create the new state, adding the default value for the new property
        let new_state = Self {
            owner_id: old_state.owner_id,
            cheddar_contract: old_state.cheddar_contract,
            game_costs: old_state.game_costs,
            user_remaining_free_games: new_user_remaining_free_games,
            user_remaining_paid_games: new_user_remaining_paid_games,
            seed_id: old_state.seed_id,
            min_deposit: old_state.min_deposit,
            ongoing_games: new_ongoing_games,
            maze_minter_contract: old_state.maze_minter_contract,
            max_game_duration: 3 * MIN_MS,
        };

        // Save the new state
        env::state_write(&new_state);

        new_state
    }

    pub fn get_contract_state(&self) -> ContractState {
        ContractState {
            owner_id: self.owner_id.to_string(),
            cheddar_contract: self.cheddar_contract.to_string(),
            maze_minter_contract: self.maze_minter_contract.to_string(),
            game_costs: self.get_games_costs(),
            seed_id: self.seed_id,
            min_deposit: self.min_deposit.to_string(),
            max_game_duration: 3 * MIN_MS,
        }
    }

    pub fn get_games_costs(&self) -> Vec<[String; 2]> {
        self.game_costs.iter().into_iter()
        .map(|(key, value)| [key.to_string(), value.to_string()])
        .collect()
    }

    // Ensure game_costs always has 1 as key, and at most 4 keys
    pub fn insert_game_cost(&mut self, key: u8, value: U128) {
        self.assert_only_owner();
        assert!(key > 0, "Key must be greater than 0");
        assert!(self.game_costs.len() < 4, "Cannot have more than 4 game costs");
        self.game_costs.insert(key, value.0);
    }

    pub fn remove_game_cost(&mut self, key: u8) {
        self.assert_only_owner();
        assert!(self.game_costs.contains_key(&key), "Key does not exist");
        self.game_costs.remove(&key);
    }

    pub fn get_cheddar_contract(&self) -> String {
        self.cheddar_contract.to_string()
    }

    pub fn set_cheddar_contract(&mut self, cheddar_contract: AccountId) {
        self.assert_only_owner();
        self.cheddar_contract = cheddar_contract;
    }

    pub fn get_user_remaining_free_games(&self, account_id: &AccountId) -> GameAmount {
        let day = get_today_day();
        log!("Getting remaining free games for {}", account_id);
        let user_free_remaining_games_data = self.user_remaining_free_games.get(account_id).unwrap_or(FreeGameInfo {
            day: 0,
            amount: 0,
        });
        log!("Day: {}", user_free_remaining_games_data.day);
        if day == user_free_remaining_games_data.day {
            user_free_remaining_games_data.amount
        } else {
            5
        }
    }

    pub fn give_free_game_to_user(&mut self, account_id: AccountId) {
        self.assert_only_owner();
        let day = env::block_timestamp_ms() / DAY_MS;

        let user_remaining_free_games = self.get_user_remaining_free_games(&account_id);

        self.user_remaining_free_games.insert(&account_id, &FreeGameInfo {
            day,
            amount: user_remaining_free_games + 1,
        });
    }

    pub fn get_user_remaining_paid_games(&self, account_id: &AccountId) -> GameAmount {
        self.user_remaining_paid_games.get(account_id).unwrap_or(0)
    }

    fn add_games_to_user(&mut self, account_id: AccountId, amount: GameAmount) {
        let user_remaining_paid_games = self.get_user_remaining_paid_games(&account_id);
        let new_remaining_paid_games = user_remaining_paid_games + amount;
        self.user_remaining_paid_games.insert(&account_id, &new_remaining_paid_games);
    }

    pub fn get_user_remaining_games(&self, account_id: &AccountId) -> (GameAmount, GameAmount) {
        (self.get_user_remaining_free_games(account_id), self.get_user_remaining_paid_games(account_id))
    }

    #[payable]
    pub fn get_seed_id(&mut self) -> SeedId {
        let account_id = env::predecessor_account_id();
        let user_ongoing_game = self.get_user_ongoing_game(account_id.clone());
        log!("User ongoing game: {:?}", user_ongoing_game.is_some());
        if user_ongoing_game.is_some() {
            log!("User has ongoing game. Losing it");
            self.lose_game();
        }

        let deposit = env::attached_deposit();
        assert!(deposit.as_yoctonear() >= self.min_deposit, "Deposit must be at least {} yoctoNEAR", self.min_deposit);

        let (remaining_free_games, remaining_paid_games) = self.get_user_remaining_games(&account_id);
        assert!(remaining_free_games > 0 || remaining_paid_games > 0, "No games remaining for the user");

        self.decrease_game(account_id.clone());
        self.seed_id += 1;
        self.ongoing_games.insert(&account_id, &Game {
            seed_id: self.seed_id,
            start_time: env::block_timestamp_ms(),
            is_ending_game: false
        });
        self.seed_id
    }

    fn lose_game(&mut self) {
        let account_id = env::predecessor_account_id();
        self.internal_end_game(account_id, U128(0), None);
    }


    fn decrease_game(&mut self, account_id: AccountId) {
        let (remaining_free_games, remaining_paid_games) = self.get_user_remaining_games(&account_id);
        if remaining_free_games > 0 {
            log!("Decreasing free game for {}", account_id);
            self.user_remaining_free_games.insert(&account_id, &FreeGameInfo {
                day: get_today_day(),
                amount: remaining_free_games - 1,
            });
        } else {
            let new_remaining_paid_games = remaining_paid_games - 1;
            self.user_remaining_paid_games.insert(&account_id, &new_remaining_paid_games);
        }
    }

    pub fn get_user_ongoing_game(&self, account_id: AccountId) -> Option<GameJson> {
        let ongoing_game = self.ongoing_games.get(&account_id).unwrap_or(Game::default());
        let now = get_now_ms();
        log!("Now: {}", get_now_ms());
        log!("Start time: {}", ongoing_game.start_time);
        assert!(now >= ongoing_game.start_time, "Start time is in the future");
        if get_now_ms() - ongoing_game.start_time >= self.max_game_duration {
            return None
        }; 
        Some(GameJson {
            seed_id: ongoing_game.seed_id,
            start_time: ongoing_game.start_time,
        })
    }

    pub fn end_game(&mut self, account_id: AccountId, amount: U128, referral: Option<AccountId>) -> Promise {
        self.assert_only_owner();
        self.internal_end_game(account_id, amount, referral)
    }

    fn internal_end_game(&mut self, account_id: AccountId, amount: U128, referral: Option<AccountId>) -> Promise {
        let ongoing_game = self.get_user_ongoing_game(account_id.clone());
        assert!(ongoing_game.is_some(), "No ongoing game for the user");
        self.ongoing_games.remove(&account_id);

        if amount > U128(0) {
            ext_maze_minter::ext(self.maze_minter_contract.clone())
                .with_static_gas(Gas::from_tgas(30))
                .with_attached_deposit(NearToken::from_yoctonear(1))
                .mint(account_id, amount, referral)
        } else {
            Promise::new(account_id)
        }
    }

    pub fn set_maze_minter_contract(&mut self, maze_minter_contract: AccountId) {
        self.assert_only_owner();
        self.maze_minter_contract = maze_minter_contract;
    }

    pub fn validate_ongoing_game(self) -> bool{
        return false
    }

    #[private]
    pub fn on_mint_callback(&self) -> String {
        match env::promise_result(0) {
            near_sdk::PromiseResult::Successful(_) => {
                env::log_str("Minting successful");
                "Minting successful".to_string()
            }
            near_sdk::PromiseResult::Failed => {
                env::log_str("Minting failed");
                "Minting failed".to_string()
            }
        }
    }

    pub fn set_max_game_duration(&mut self, game_duration_seconds: u64) {
        self.assert_only_owner();
        self.max_game_duration = game_duration_seconds * 1000;
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use near_sdk::{test_utils::{accounts, VMContextBuilder}, testing_env, NearToken};

    use super::*;

    const MS_TO_NANOS: u64 = 1_000_000;

    fn setup_contract() -> (VMContextBuilder, MazeGameBuyerContract) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).block_timestamp(DAY_MS * MS_TO_NANOS).build());
        let cheddar_contract = AccountId::from_str("token.cheddar.near").unwrap();
        let maze_minter_contract = AccountId::from_str("minter.near").unwrap();
        let contract = MazeGameBuyerContract::new(cheddar_contract, maze_minter_contract);
        (context, contract)
    }

    #[test]
    fn get_default_game_costs() {
        let (_, contract) = setup_contract();
        // this test did not call set_greeting so should return the default "Hello" greeting
        let game_costs = [["1".to_string(), to_yocto_u8(15).0.to_string()], ["10".to_string(), to_yocto_u8(14).0.to_string()]];
        assert_eq!(contract.get_games_costs(), game_costs);
    }

    #[test]
    fn set_then_get_game_costs() {
        let (_, mut contract) = setup_contract();
        contract.insert_game_cost(1, to_yocto_u8(20));
        let new_game_costs_1 = [["1".to_string(), to_yocto_u8(20).0.to_string()], ["10".to_string(), to_yocto_u8(14).0.to_string()]];
        assert_eq!(contract.get_games_costs(), new_game_costs_1);

        contract.remove_game_cost(10);
        let new_game_costs_2 = [["1".to_string(), to_yocto_u8(20).0.to_string()]];
        assert_eq!(contract.get_games_costs(), new_game_costs_2);
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
        assert_eq!(contract.get_user_remaining_free_games(&user), 5);
    }

    #[test]
    fn set_then_get_free_games() {
        let (_, mut contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        contract.give_free_game_to_user(user.clone());
        assert_eq!(contract.get_user_remaining_free_games(&user), 6);
    }

    #[test]
    fn get_free_games_on_new_day() {
        let (mut context, mut contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        contract.give_free_game_to_user(user.clone());
        testing_env!(context.block_timestamp(2 * DAY_MS * MS_TO_NANOS).build());
        assert_eq!(contract.get_user_remaining_free_games(&user), 5);
    }

    #[test]
    fn get_paid_games() {
        let (_, contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        assert_eq!(contract.get_user_remaining_paid_games(&user), 0);
    }

    #[test]
    fn get_remaining_games() {
        let (_, contract) = setup_contract();
        let user = AccountId::from_str("test.near").unwrap();
        assert_eq!(contract.get_user_remaining_games(&user), (5, 0));
    }

    #[test]
    fn get_seed_id() {
        let (mut context, mut contract) = setup_contract();
        context.attached_deposit(NearToken::from_yoctonear(1_000_000_000_000_000_000_000));
        testing_env!(context.build());
        let user = accounts(0);
        assert_eq!(contract.get_seed_id(), 1);
        assert_eq!(contract.get_user_remaining_games(&user), (4, 0));
    }

    #[test]
    fn get_seed_id_assume_lose_get_new_seed_id() {
        let (mut context, mut contract) = setup_contract();
        context.attached_deposit(NearToken::from_yoctonear(1_000_000_000_000_000_000_000));
        testing_env!(context.build());
        let user = accounts(0);
        assert_eq!(contract.get_seed_id(), 1);
        assert_eq!(contract.get_user_remaining_games(&user), (4, 0));

        assert_eq!(contract.get_seed_id(), 2);
        assert_eq!(contract.get_user_remaining_games(&user), (3, 0));
    }

    #[test]
    fn win_and_restart() {
        let (mut context, mut contract) = setup_contract();
        context.attached_deposit(NearToken::from_yoctonear(1_000_000_000_000_000_000_000));
        testing_env!(context.build());
        let user = accounts(0);
        assert_eq!(contract.get_seed_id(), 1);
        assert_eq!(contract.get_user_remaining_games(&user), (4, 0));

        contract.end_game(user.clone(), U128(1_000_000_000_000_000_000_000_000), None);
        assert_eq!(contract.get_seed_id(), 2);
        assert_eq!(contract.get_user_remaining_games(&user), (3, 0));
    }

    #[test]
    fn test_ongoing_game() {
        let (mut context, mut contract) = setup_contract();
        context.attached_deposit(NearToken::from_yoctonear(1_000_000_000_000_000_000_000));
        testing_env!(context.build());
        let user = accounts(0);
        let ongoing_game = contract.get_user_ongoing_game(user.clone());
        assert!(ongoing_game.is_none());
        contract.get_seed_id();
        let ongoing_game = contract.get_user_ongoing_game(user.clone());
        assert!(ongoing_game.is_some());
        assert!(ongoing_game.unwrap().seed_id == 1);
    }

    #[test]
    fn set_then_get_maze_minter_contract() {
        let (_, mut contract) = setup_contract();
        let new_maze_minter_contract = AccountId::from_str("new.maze.minter.near").unwrap();
        contract.set_maze_minter_contract(new_maze_minter_contract.clone());
        let contract_state = contract.get_contract_state();
        assert!(contract_state.maze_minter_contract == new_maze_minter_contract);
    }

    #[test]
    fn ensure_game_lost_after_time() {
        let (mut context, mut contract) = setup_contract();
        context.attached_deposit(NearToken::from_yoctonear(1_000_000_000_000_000_000_000));
        testing_env!(context.build());
        let user = accounts(0);

        assert_eq!(contract.get_seed_id(), 1);
        let ongoing_game = contract.get_user_ongoing_game(user.clone());
        assert!(ongoing_game.is_some());
        let unwraped_ongoing_game = ongoing_game.unwrap();
        assert!(unwraped_ongoing_game.seed_id == 1);
        assert!(unwraped_ongoing_game.start_time > 0);

        let state = contract.get_contract_state();
        let new_time = (get_now_ms() + state.max_game_duration) * MS_TO_NANOS;
        testing_env!(context.predecessor_account_id(accounts(0)).block_timestamp(new_time).build());

        let ongoing_game = contract.get_user_ongoing_game(user.clone());
        assert!(ongoing_game.is_none());
    }

}
