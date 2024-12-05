use near_sdk::{ext_contract, json_types::U128, AccountId};

// External contract interface for the Maze Minter contract
#[ext_contract(ext_maze_minter)]
pub trait ExtMazeMinter {
    fn mint(&mut self, recipient: AccountId, amount: U128, referral: Option<AccountId>) -> (u128, u128);
}