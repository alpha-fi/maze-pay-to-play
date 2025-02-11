# export ACC=maze-buyer.testnet
# bash scripts/redeploy.sh maze-buyer.testnet
set +e

bash scripts/build.sh

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <account_id>"
    exit 1
fi

ACCOUNT_ID=$1 # maze-buyer.testnet

echo Redeploying contract on account $ACCOUNT_ID

near deploy $ACCOUNT_ID ./res/target/maze_game_buyer_contract.wasm
if [ $? -ne 0 ]; then # $? is the exit code of the last command. 0 means success
    exit 1
fi
echo Successfully redeployed contract on account $ACCOUNT_ID.