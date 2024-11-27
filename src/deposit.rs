use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::{near_bindgen, AccountId, json_types::U128, env, PromiseOrValue};
use crate::utils::safe_u128_to_u16;
use crate::MazeGameBuyerContractExt;
use crate::MazeGameBuyerContract;

#[allow(unused_variables)]
#[near_bindgen]
impl FungibleTokenReceiver for MazeGameBuyerContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        #[allow(unused_variables)]
        msg: String,
    ) -> PromiseOrValue<U128> {
        let ft_token = env::predecessor_account_id();
        assert!(ft_token == self.cheddar_contract, "Only cheddar is accepted {}", self.cheddar_contract);
        let mut game_promo_num = 0;
        // game_costs should always be limited to 4 key-value pairs
        for (key, value) in self.game_costs.into_iter() {
            if amount.0 / *key as u128 >= *value {
                game_promo_num = *key;
            } else {
                break;
            }
        }
        let single_game_cost = self.game_costs.get(&1).unwrap();
        assert!(game_promo_num != 0, "Insufficient cheddar sent {}. Sent at least {} cheddar", amount.0, single_game_cost);
        if game_promo_num == 0 {
            return PromiseOrValue::Value(amount);
        }
        let game_cost = self.get_game_cost(game_promo_num);
        let games_bought = amount.0 / game_cost;
        let games_bought_u16 = safe_u128_to_u16(games_bought).expect(format!("Too many games bought. Limit is {}", u16::MAX).as_str());
        self.add_games_to_user(sender_id.clone(), games_bought_u16);

        let remaining_cheddar = amount.0 % game_cost;
        PromiseOrValue::Value(U128::from(remaining_cheddar))

    }

}