cargo build --target wasm32-unknown-unknown --release
mkdir -p res/target # Creates dir if not exists
cp ./target/wasm32-unknown-unknown/release/maze_game_buyer_contract.wasm ./res/target/maze_game_buyer_contract.wasm