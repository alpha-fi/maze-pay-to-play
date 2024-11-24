
use crate::*;

impl MazeGameBuyerContract {
    
    pub(crate) fn assert_only_owner(&self) {
        require!(
            self.owner_id == env::predecessor_account_id(),
            "Only the owner can call this function."
        );
    }

    pub(crate) fn get_game_cost(&self, game_promo_num: u8) -> Balance {
        self.game_costs.get(&game_promo_num).expect("Game cost not found").clone()
    }
    
}