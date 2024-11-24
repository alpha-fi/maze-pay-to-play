use std::str::FromStr;

use near_sdk::{log, AccountId};
use near_workspaces::{types::NearToken, Account, Contract};
use serde_json::json;
use near_units::parse_near;


const FIVE_NEAR: NearToken = NearToken::from_near(5);

async fn get_root() -> Account {
    let sandbox = near_workspaces::sandbox().await.unwrap();
    let root = sandbox.root_account().unwrap();
    
    root
    // let (buyer_contract, owner_account) = initialize_contract(root).await;
    // let token_cheddar_contract = initialize_cheddar_token_contract(root).await;

    // (buyer_contract, owner_account, token_cheddar_contract)
}

async fn initialize_contract(root: Account) -> (Contract, Account) {
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

async fn initialize_cheddar_token_contract(root: Account) -> Contract {
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
    let root = get_root().await;
    let (contract, user_account) = initialize_contract(root).await;
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
    let root = get_root().await;
    let (contract, user_account) = initialize_contract(root).await;
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

// #[tokio::test]
// async fn test_free_games() -> Result<(), Box<dyn std::error::Error>> {
//     let (contract, user_account) = initialize_contract().await;
//     let user = AccountId::from_str("test.near").unwrap();
//     let outcome: near_workspaces::result::ExecutionFinalResult = user_account
//         .call(contract.id(), "get_user_remaining_free_games")
//         .args_json(json!({"account_id": &user}))
//         .transact()
//         .await?;
//     if outcome.is_failure() {
//         log!("Error calling set_cheddar_contract: {:?}", outcome);
//     }
//     assert!(outcome.is_success());

//     let contract_new_cheddar_contract: serde_json::Value = user_account
//         .view(contract.id(), "get_cheddar_contract")
//         .args_json(json!({}))
//         .await?
//         .json()?;
//     log!("contract_new_cheddar_contract: {:?}", contract_new_cheddar_contract);
//     assert_eq!(contract_new_cheddar_contract, user.to_string());

//     Ok(())
// }