use std::{str::FromStr, time::Instant};

use near_sdk::{log, AccountId};
use near_workspaces::{network::Sandbox, types::NearToken, Account, Contract, Worker};
use serde_json::json;
use near_units::parse_near;


const FIVE_NEAR: NearToken = NearToken::from_near(5);
const STORAGE_DEPOSIT_AMOUNT: NearToken = NearToken::from_near(1);

async fn get_root() -> (Account, Worker<Sandbox>) {
    let sandbox = near_workspaces::sandbox().await.unwrap();
    let root = sandbox.root_account().unwrap();
    
    (root, sandbox)
    // let (buyer_contract, owner_account) = initialize_contract(root).await;
    // let token_cheddar_contract = initialize_cheddar_token_contract(root).await;

    // (buyer_contract, owner_account, token_cheddar_contract)
}

async fn initialize_contract(root: &Account) -> (Contract, Account) {
    // let sandbox = near_workspaces::sandbox().await.unwrap();
    let contract_wasm = near_workspaces::compile_project("./").await.unwrap();

    // let root = sandbox.root_account().unwrap();
    let user_account = root.create_subaccount("user").transact().await.unwrap().unwrap();
    let contract_account = root.create_subaccount("contract").initial_balance(FIVE_NEAR).transact().await.unwrap().unwrap();

    let contract = contract_account.deploy(&contract_wasm).await.unwrap().unwrap();
    let outcome = user_account
        .call(contract.id(), "new")
        .args_json(json!({"cheddar_contract": "token.cheddar.near"}))
        .transact()
        .await.unwrap();
    if outcome.is_failure() {
        log!("Error calling new: {:?}", outcome);
    }
    assert!(outcome.is_success());
    (contract, user_account)
}

async fn initialize_cheddar_token_contract(root: &Account) -> Contract {
    let contract_wasm_path = "../res/fungible_token.wasm";

    let cheddar_account = root.create_subaccount("cheddar").transact().await.unwrap().unwrap(); 
    let token_cheddar_account = cheddar_account.create_subaccount("token").transact().await.unwrap().unwrap(); 
    // let token_cheddar_account = cheddar_account.cre
    let token_cheddar_contract = token_cheddar_account.deploy(&std::fs::read(contract_wasm_path).unwrap()).await.unwrap().unwrap();

    let outcome = cheddar_account
        .call(token_cheddar_contract.id(),"new_default_meta") // NEP-141 initialization method
        .args_json(json!({
            "owner_id": cheddar_account.id(),
            "total_supply": parse_near!("1000 N")
        }))
        .transact()
        .await
        .unwrap();

    if outcome.is_failure() {
        log!("Error calling new: {:?}", outcome);
    }
    assert!(outcome.is_success());

    println!("Cheddar contract initialized!");

    token_cheddar_contract
}

#[tokio::test]
async fn test_contract_game_costs() -> Result<(), Box<dyn std::error::Error>> {
    let (root, _) = get_root().await;
    let (contract, user_account) = initialize_contract(&root).await;
    let new_game_costs = json!({"1": 20, "10": 15});
    let outcome = user_account
        .call(contract.id(), "set_game_costs")
        .args_json(json!({"game_costs": &new_game_costs}))
        .transact()
        .await?;
    if outcome.is_failure() {
        log!("Error calling set_game_costs: {:?}", outcome);
    }
    assert!(outcome.is_success());

    let contract_new_game_costs: serde_json::Value = user_account
        .view(contract.id(), "get_games_costs")
        .args_json(json!({}))
        .await?
        .json()?;
    assert_eq!(contract_new_game_costs, new_game_costs);

    Ok(())
}

#[tokio::test]
async fn test_cheddar_token() -> Result<(), Box<dyn std::error::Error>> {
    let (root, _) = get_root().await;
    let (contract, user_account) = initialize_contract(&root).await;
    let new_cheddar_contract = AccountId::from_str("token-v2.cheddar.near").unwrap();
    let outcome: near_workspaces::result::ExecutionFinalResult = user_account
        .call(contract.id(), "set_cheddar_contract")
        .args_json(json!({"cheddar_contract": &new_cheddar_contract}))
        .transact()
        .await?;
    if outcome.is_failure() {
        log!("Error calling set_cheddar_contract: {:?}", outcome);
    }
    assert!(outcome.is_success());

    let contract_new_cheddar_contract: serde_json::Value = user_account
        .view(contract.id(), "get_cheddar_contract")
        .args_json(json!({}))
        .await?
        .json()?;
    log!("contract_new_cheddar_contract: {:?}", contract_new_cheddar_contract);
    assert_eq!(contract_new_cheddar_contract, new_cheddar_contract.to_string());

    Ok(())
}

#[tokio::test]
#[ignore = "This test takes a long time to run due fast forwards. Comment ignore when necessary. Recommend to run with -- --nocapture"]
async fn test_free_games() -> Result<(), Box<dyn std::error::Error>> {
    let (root, sandbox) = get_root().await;
    let (contract, owner_account) = initialize_contract(&root).await;
    let user = root.create_subaccount("test").transact().await.unwrap().unwrap();

    let initial_free_games: u16 = user
        .view(contract.id(), "get_user_remaining_free_games")
        .args_json(json!({"account_id": &user.id()}))
        .await?
        .json()?;
    
    assert_eq!(initial_free_games, 5);

    let outcome_give_free_game: near_workspaces::result::ExecutionFinalResult = owner_account
        .call(contract.id(), "give_free_game_to_user")
        .args_json(json!({"account_id": &user.id()}))
        .transact()
        .await?;
    if outcome_give_free_game.is_failure() {
        log!("Error calling give_free_game_to_user: {:?}", outcome_give_free_game);
    }
    assert!(outcome_give_free_game.is_success());

    let current_block = sandbox.view_block().await?;

    // Define blocks per day (1 block per second)
    let nanos_per_day = 86_400 * 1_000 * 1_000 * 1_000;

    let mut current_timestamp = current_block.timestamp();
    let day_today = current_timestamp / nanos_per_day;
    let start = Instant::now();
    log!("day today: {}", day_today);
    while current_timestamp / nanos_per_day == day_today  {
        let mid2_free_games: u16 = user
            .view(contract.id(), "get_user_remaining_free_games")
            .args_json(json!({"account_id": &user.id()}))
            .await?
            .json()?;

        assert_eq!(mid2_free_games, 6);
        let _ = sandbox.fast_forward(60 * 60).await;
        let current_block = sandbox.view_block().await?;
        current_timestamp = current_block.timestamp();
        log!("current_timestamp: {}", current_timestamp);
    }
    let duration = start.elapsed();
    
    log!("Advancing a day took: {:?} secs", duration.as_secs());
    log!("day today 2: {}", current_timestamp / nanos_per_day);

    let final_free_games: u16 = user
        .view(contract.id(), "get_user_remaining_free_games")
        .args_json(json!({"account_id": &user.id()}))
        .await?
        .json()?;

    assert_eq!(final_free_games, 5);

    Ok(())
}

// #[tokio::test]
// async fn test_buy_game() -> Result<(), Box<dyn std::error::Error>> {
//     let (root, _) = get_root().await;
//     let (contract, owner_account) = initialize_contract(&root).await;
//     let cheddar_token_contract = initialize_cheddar_token_contract(&root).await;
    
//     let user = root.create_subaccount("test").initial_balance(FIVE_NEAR).transact().await.unwrap().unwrap();
//     let storage_deposit_result = user.call(cheddar_token_contract.id(), "storage_deposit")
//         .args_json(json!({}))
//         .deposit(STORAGE_DEPOSIT_AMOUNT)
//         .transact()
//         .await?;

//     if storage_deposit_result.is_failure() {
//         log!("Error calling storage_deposit: {:?}", storage_deposit_result);
//     }
//     assert!(storage_deposit_result.is_success());

//     let transfer_result = cheddar_token_contract.call("ft_transfer")
//         .args_json(json!({
//             "receiver_id": &user.id(),
//             "amount": parse_near!("100 N").to_string()
//         }))
//         .deposit(NearToken::from_yoctonear(1))
//         .transact()
//         .await?;

//     if transfer_result.is_failure() {
//         log!("Error calling ft_transfer from contract to user: {:?}", transfer_result);
//     }
//     assert!(transfer_result.is_success());

//     let initial_paid_games: u16 = user
//         .view(contract.id(), "get_user_remaining_paid_games")
//         .args_json(json!({"account_id": &user.id()}))
//         .await?
//         .json()?;

//     assert_eq!(initial_paid_games, 0);

//     let transfer_call_result = user.call(cheddar_token_contract.id(), "ft_transfer_call")
//         .args_json(json!({
//             "receiver_id": &contract.id(),
//             "amount": parse_near!("15 N").to_string(),
//             "msg": ""
//         }))
//         .deposit(NearToken::from_yoctonear(1))
//         .transact()
//         .await?;

//     if transfer_call_result.is_failure() {
//         log!("Error calling ft_transfer from user to buyer: {:?}", transfer_call_result);
//     }
//     assert!(transfer_call_result.is_success());

//     let mid_paid_games: u16 = user
//         .view(contract.id(), "get_user_remaining_paid_games")
//         .args_json(json!({"account_id": &user.id()}))
//         .await?
//         .json()?;

//     assert_eq!(mid_paid_games, 1);

//     let transfer_call_result = user.call(cheddar_token_contract.id(), "ft_transfer_call")
//         .args_json(json!({
//             "receiver_id": &contract.id(),
//             "amount": parse_near!("140 N").to_string(),
//             "msg": ""
//         }))
//         .deposit(NearToken::from_yoctonear(1))
//         .transact()
//         .await?;

//     if transfer_call_result.is_failure() {
//         log!("Error calling ft_transfer from user to buyer: {:?}", transfer_call_result);
//     }
//     assert!(transfer_call_result.is_success());

//     let final_paid_games: u16 = user
//         .view(contract.id(), "get_user_remaining_paid_games")
//         .args_json(json!({"account_id": &user.id()}))
//         .await?
//         .json()?;

//     assert_eq!(final_paid_games, 11);
   
//     Ok(())
// }