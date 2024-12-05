# export ACC=maze-buyer.testnet
# export FN=migrate
# export ARGS={}
set +e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <account_id> <migrate_function> <migrate_args_string>"
    exit 1
fi

ACCOUNT_ID=$1 # maze-buyer.testnet
MIGRATE_FUNCTION=$2 # migrate
MIGRATE_ARGS_STRING=$3 # {}

echo Redeploying contract on account $ACCOUNT_ID with migrate function $MIGRATE_FUNCTION and args $MIGRATE_ARGS_STRING

near deploy $ACCOUNT_ID ./res/target/maze_game_buyer_contract.wasm
if [ $? -ne 0 ]; then # $? is the exit code of the last command. 0 means success
    exit 1
fi
echo Successfully redeployed contract on account $ACCOUNT_ID. Starting migration...

near call $ACCOUNT_ID $MIGRATE_FUNCTION $MIGRATE_ARGS_STRING --accountId $ACCOUNT_ID