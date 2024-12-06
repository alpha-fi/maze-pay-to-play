# bash scripts/delete_recreate_deploy.sh maze-buyer.testnet token-v3.cheddar.testnet maze1.cheddar.testnet
set +e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <account_id> <cheddar_contract> <minter_contract>"
    exit 1
fi

ACCOUNT_ID=$1 # maze-buyer.testnet
CHEDDAR_CONTRACT=$2 # token-v3.cheddar.testnet
MINTER_CONTRACT=$3 # maze1.cheddar.testnet

# near delete $ACCOUNT_ID silkking.testnet
# near create-account $ACCOUNT_ID --useFaucet
bash ./scripts/deploy.sh $ACCOUNT_ID $CHEDDAR_CONTRACT $MINTER_CONTRACT