USER_PEM="/home/flaerd/Flavius/licenta/dev-wallet-owner.pem"
PROXY="https://devnet-gateway.multiversx.com"
CHAIN_ID="D"

WASM_PATH="/home/flaerd/Flavius/licenta/MultiversX/nft-stake-contract/output/nft-stake-contract.wasm"

USER_ADDRESS=erd15y7qz4j0czthe3cv35zq6kejssz0n8xw3j3etan5hr9jt6lea0wsnr2plm

# last sc addr  erd1qqqqqqqqqqqqqpgq0pkumrkgc67ms0tr5huwf0uuvyfzhhdsa0wsu0043g
SC_ADDRESS=erd1qqqqqqqqqqqqqpgqsn4q75yt0lf4822k0plldnxjet49vkpua0ws6un5tz
STAKE_AMOUNT=50000000000000000 # => 0.05XEGLD
UNSTAKE_AMOUNT=50000000000000000 # => 0.05XEGLD
SC_FUNDING_CAP=500000000000000000 # => 0.5 XEGLD
NFT_VALUE=500000000000000000 # => 0.50XEGLD
NFT_TOKEN_IDENTIFIER=str:NFTTEST-40caee # NFTTEST-40caee
NFT_NOUNCE=01

USER_WALLET="$(mxpy wallet pem-address $USER_PEM)"

deploy() {
    mxpy --verbose contract deploy --project=${PROJECT} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=50000000 \
    --send --outfile="deploy-devnet.interaction.json" \
    --proxy=${PROXY} --chain=${CHAIN_ID} \
    --arguments ${NFT_VALUE} ${NFT_TOKEN_IDENTIFIER} || return
}

upgrade() {
    mxpy --verbose contract upgrade ${SC_ADDRESS} \
    --bytecode=${WASM_PATH} \
    --recall-nonce --pem=${USER_PEM} \
    --gas-limit=50000000 \
    --send --outfile="upgrade-devnet.interaction.json" \
    --proxy=${PROXY} --chain=${CHAIN_ID} \
    --arguments ${NFT_VALUE} ${NFT_TOKEN_IDENTIFIER} || return
}

addRewards() {
    mxpy --verbose contract call ${SC_ADDRESS} \
    --proxy=${PROXY} --chain=${CHAIN_ID} \
    --send --recall-nonce --pem=${USER_PEM} \
    --gas-limit=10000000 \
    --value=${SC_FUNDING_CAP} \
    --function="add_rewards"
}

stakeNFT() {
    method_name=str:stake_nft

    mxpy --verbose contract call ${USER_ADDRESS} \
    --proxy=${PROXY} --chain=${CHAIN_ID} \
    --send --recall-nonce --pem=${USER_PEM} \
    --gas-limit=10000000 \
    --function="ESDTNFTTransfer" \
    --arguments ${NFT_TOKEN_IDENTIFIER} ${NFT_NOUNCE} 1 ${SC_ADDRESS} ${method_name} \
    --send || return
}

unstakeNFT() {
    mxpy --verbose contract call ${SC_ADDRESS} \
    --proxy=${PROXY} --chain=${CHAIN_ID} \
    --send --recall-nonce --pem=${USER_PEM} \
    --gas-limit=10000000 \
    --function="unstakeNFTS"
}

########## GETs ##########

getFundingCap() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getFundingCap"
}

getCollectionIdentifier() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getNFTIdentifier"
}

getCalculatedRewards() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getCalculatedRewards" \
    --arguments ${USER_ADDRESS}
}

getNFTNonces() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getNftNonces" \
    --arguments ${USER_ADDRESS}
}

getLockedEpoch() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getLockedEpoch" \
    --arguments ${USER_ADDRESS}
}

getLastClaimEpoch() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getLastClaimEpoch" \
    --arguments ${USER_ADDRESS}
}

getRewards() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getRewards" \
    --arguments ${USER_ADDRESS}
}

getNFTValue() {
    mxpy --verbose contract query ${SC_ADDRESS} \
    --proxy=${PROXY} \
    --function="getNFTValue"
}
