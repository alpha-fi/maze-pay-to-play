# export ACC=maze-buyer.testnet
# export CH=token-v3.cheddar.testnet
# export MI=maze1.cheddar.testnet
set +e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <account_id> <cheddar_contract> <minter_contract>"
    exit 1
fi

ACCOUNT_ID=$1 # maze-buyer.testnet
CHEDDAR_CONTRACT=$2 # token-v3.cheddar.testnet
MINTER_CONTRACT=$3 # maze1.cheddar.testnet

echo Deploying contract on account $ACCOUNT_ID with cheddar contract $CHEDDAR_CONTRACT and minter contract $MINTER_CONTRACT
echo '{"cheddar_contract": "'$CHEDDAR_CONTRACT'", "maze_minter_contract": "'$MINTER_CONTRACT'"}'

near deploy $ACCOUNT_ID ./res/target/maze_game_buyer_contract.wasm \
    --initFunction new \
    --initArgs '{"cheddar_contract": "'$CHEDDAR_CONTRACT'", "maze_minter_contract": "'$MINTER_CONTRACT'"}' \
    --verbose