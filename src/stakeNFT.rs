#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use multiversx_sc::types::heap::Vec;

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const BLOCKS_IN_YEAR: u64 = (60 * 60 * 24 * 365) / 6;
pub const APY: u64 = 1_000; // 10%

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct NFTStakeInfo<M: ManagedTypeApi> {
    pub locked_epoch: u64,
    pub last_claim_epoch: u64,
    pub nft_nonces: Vec<u64>,
    pub rewards: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait NftStakeContract {
    #[init]
    fn init(&self, nft_value: BigUint, nft_identifier: EgldOrEsdtTokenIdentifier) {
        self.nft_value().set(nft_value);
        self.collection_nft_identifier().set_if_empty(&nft_identifier);
    }

    #[payable("EGLD")]
    #[only_owner]
    #[endpoint]
    fn add_rewards(&self) {
        let added_amount = self.call_value().egld_value();
        require!(added_amount > 0, "Must pay more than 0");

        self.funding_cap().update(|funding_cap| {
            *funding_cap += added_amount;
        });
    }

    #[payable("*")]
    #[endpoint]
    fn stake_nft(&self) {
        let nfts = self.call_value().all_esdt_transfers();
        let _sc_caller = self.blockchain().get_caller();
        let _current_block = self.blockchain().get_block_nonce();
        let _stake_mapper = self.nft_stake_info(&_sc_caller);
        let mut nonce_vector: Vec<u64> = Vec::new();

        for nft in &nfts {
            require!(
                nft.token_identifier == self.collection_nft_identifier().get(),
                "This NFT doesn't belong to this collection"
            );
            require!(nft.token_nonce != 0, "Invalid nft nonce");
            require!(nft.amount == BigUint::from(1u32), "You can only send 1 nft");
            // We alredy know the collection identifier so we only need to store the nounce of the sent nfts
            nonce_vector.push(nft.token_nonce);
        }

        let mut staking_pos = if !self.nft_stake_info(&_sc_caller).is_empty() {
            _stake_mapper.get()
        } else {
            NFTStakeInfo {
                locked_epoch: _current_block,
                last_claim_epoch: _current_block,
                nft_nonces: nonce_vector.clone(),
                rewards: BigUint::from(0u64),
            }
        };

        self.claim_rewards_for_user(&_sc_caller, &mut staking_pos);

        //Add the old nounces to the new ones
        for nonce in staking_pos.nft_nonces {
            &nonce_vector.push(nonce);
        }
        staking_pos.nft_nonces = nonce_vector;
        _stake_mapper.set(&staking_pos);
    }

    #[endpoint(unstakeNFTS)]
    fn unstake(&self) {
        let _sc_caller = self.blockchain().get_caller();
        require!(!self.nft_stake_info(&_sc_caller).is_empty(), "You didn't stake");

        let _current_block = self.blockchain().get_block_nonce();
        let mut _staking_pos = self.nft_stake_info(&_sc_caller).get();
        let _nft_identifier = self.collection_nft_identifier().get();
        let _amount = BigUint::from(1u64);

        self.claim_rewards_for_user(&_sc_caller, &mut _staking_pos);

        for nft_nonce in _staking_pos.nft_nonces {
            self.send().direct(&_sc_caller, &_nft_identifier, nft_nonce, &_amount);
        }

        self.nft_stake_info(&_sc_caller).clear();
    }

    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        let _sc_caller = self.blockchain().get_caller();
        let stake_mapper = self.nft_stake_info(&_sc_caller);

        let mut staking_pos = stake_mapper.get();
        self.claim_rewards_for_user(&_sc_caller, &mut staking_pos);
        stake_mapper.set(&staking_pos);
    }

    fn claim_rewards_for_user(
        &self,
        user: &ManagedAddress,
        staking_pos: &mut NFTStakeInfo<Self::Api>
    ) {
        let reward_amount = self.calculate_rewards(staking_pos);
        let current_block = self.blockchain().get_block_nonce();
        staking_pos.last_claim_epoch = current_block;

        if reward_amount > 0 {
            self.send().direct_egld(user, &reward_amount);
        }
    }

    fn calculate_rewards(&self, staking_position: &NFTStakeInfo<Self::Api>) -> BigUint {
        let current_block = self.blockchain().get_block_nonce();
        if current_block <= staking_position.last_claim_epoch {
            return BigUint::zero();
        }

        let block_diff = current_block - staking_position.last_claim_epoch;

        (((self.nft_value().get() * BigUint::from(staking_position.nft_nonces.len()) * APY) /
            MAX_PERCENTAGE) *
            block_diff) /
            BLOCKS_IN_YEAR
    }

    // VIEW

    #[view(getCalculatedRewards)]
    fn get_calculated_rewards(&self, address: &ManagedAddress) -> BigUint<Self::Api> {
        require!(!self.nft_stake_info(&address).is_empty(), "You didn't stake");
        let nft_stake = self.nft_stake_info(&address).get();

        return self.calculate_rewards(&nft_stake);
    }

    #[view(getNftNonces)]
    fn get_nft_nonces(&self, address: &ManagedAddress) -> usize {
        require!(!self.nft_stake_info(&address).is_empty(), "You didn't stake");
        let nft_stake = self.nft_stake_info(&address).get();
        return nft_stake.nft_nonces.len();
    }

    #[view(getLockedEpoch)]
    fn get_locked_epoch(&self, address: &ManagedAddress) -> u64 {
        require!(!self.nft_stake_info(&address).is_empty(), "You didn't stake");
        let nft_stake = self.nft_stake_info(&address).get();
        return nft_stake.locked_epoch;
    }

    #[view(getLastClaimEpoch)]
    fn get_last_claim_epoch(&self, address: &ManagedAddress) -> u64 {
        require!(!self.nft_stake_info(&address).is_empty(), "You didn't stake");
        let nft_stake = self.nft_stake_info(&address).get();
        return nft_stake.last_claim_epoch;
    }

    // STORAGE

    #[view(getFundingCap)]
    #[storage_mapper("fundingCap")]
    fn funding_cap(&self) -> SingleValueMapper<BigUint>;

    #[view(getNFTIdentifier)]
    #[storage_mapper("collection_nft_identifier")]
    fn collection_nft_identifier(&self) -> SingleValueMapper<EgldOrEsdtTokenIdentifier>;

    #[view(getNFTStakeInfo)]
    #[storage_mapper("nft_stake_info")]
    fn nft_stake_info(
        &self,
        address: &ManagedAddress
    ) -> SingleValueMapper<NFTStakeInfo<Self::Api>>;

    #[view(getNFTValue)]
    #[storage_mapper("nft_value")]
    fn nft_value(&self) -> SingleValueMapper<BigUint>;

    #[view(getApy)]
    #[storage_mapper("apy")]
    fn apy(&self) -> SingleValueMapper<u64>;
}