use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::SeedId;

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct GameJson {
	pub seed_id: SeedId,
	pub start_time: u64,
}