#export NODE_URL="http://3.14.48.188:7777/rpc"
export DEPLOY_TIMEOUT="120000"
export NODE_URL="http://127.0.0.1:7777/rpc"
export NETWORK_NAME=casper-net-1
export EVENT_STREAM_ADDRESS="http://127.0.0.1:19999/events"
export PRIVATE_KEY_1=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-1/secret_key.pem  | sed -n '2 p'`
export PRIVATE_KEY_2=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-2/secret_key.pem  | sed -n '2 p'`
export PRIVATE_KEY_3=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-3/secret_key.pem  | sed -n '2 p'`
export PRIVATE_KEY_4=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-4/secret_key.pem  | sed -n '2 p'`
#export PRIVATE_KEY_1=`cat /home/casperlabs-dev/Downloads/devnet_user_1_secret.pem  | sed -n '2 p'`
#export PRIVATE_KEY_2=`cat /home/casperlabs-dev/Downloads/devnet_user_2_secret.pem  | sed -n '2 p'`
#export PRIVATE_KEY_3=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-3/secret_key.pem  | sed -n '2 p'`
#export PRIVATE_KEY_4=`cat /home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-4/secret_key.pem  | sed -n '2 p'`
 
#export MASTER_KEY_PAIR_PATH=/home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-1
#export USER1_KEY_PAIR_PATH=/home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-2
#export USER2_KEY_PAIR_PATH=/home/casperlabs-dev/DEV/src/casper-nctl/assets/net-1/users/user-3
npm run generate:wasm
npm run test:e2e
