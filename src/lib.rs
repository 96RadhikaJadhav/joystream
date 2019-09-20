// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;

use codec::{Decode, Encode};
use runtime_primitives::traits::AccountIdConversion;
use runtime_primitives::ModuleId;
use srml_support::traits::{Currency, ExistenceRequirement, Get, WithdrawReason};
use srml_support::{decl_module, decl_storage, ensure, StorageMap, StorageValue};

use rstd::collections::btree_map::BTreeMap;
use system;

mod mock;
mod tests;

pub type StakeId = u64;
pub const FIRST_STAKE_ID: u64 = 1;

pub type SlashId = u64;
pub const FIRST_SLASH_ID: u64 = 1;

pub type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait + Sized {
    /// The currency that is managed by the module
    type Currency: Currency<Self::AccountId>;

    /// ModuleId for computing deterministic AccountId for the module
    type StakePoolId: Get<[u8; 8]>;

    /// Type that will handle various staking events
    type StakingEventsHandler: StakingEventsHandler<Self>;
}

pub trait StakingEventsHandler<T: Trait> {
    // Type of handler which handles unstaking event.
    fn unstaked(id: StakeId);

    // Type of handler which handles slashing event.
    // NB: actually_slashed can be less than amount of the slash itself if the
    // claim amount on the stake cannot cover it fully.
    fn slashed(id: StakeId, slash_id: SlashId, actually_slashed: BalanceOf<T>);
}

impl<T: Trait> StakingEventsHandler<T> for () {
    fn unstaked(_id: StakeId) {}
    fn slashed(_id: StakeId, _slash_id: SlashId, _actually_slashed: BalanceOf<T>) {}
}

// Helper so we can provide multiple handlers
impl<T: Trait, X: StakingEventsHandler<T>, Y: StakingEventsHandler<T>> StakingEventsHandler<T>
    for (X, Y)
{
    fn unstaked(id: StakeId) {
        X::unstaked(id);
        Y::unstaked(id);
    }
    fn slashed(id: StakeId, slash_id: SlashId, actually_slashed: BalanceOf<T>) {
        X::slashed(id, slash_id, actually_slashed);
        Y::slashed(id, slash_id, actually_slashed);
    }
}

#[derive(Encode, Decode, Debug, Default, Eq, PartialEq)]
pub struct Slash<BlockNumber, Balance> {
    // rename to SlashingState ?

    // The block slashing was initiated.
    started_at_block: BlockNumber,

    // Whether slashing is in active, or conversley paused state
    // Blocks are only counted towards slashing execution delay when active.
    is_active: bool,

    // The number blocks which must be finalised while in the active period before the slashing can be executed
    blocks_remaining_in_active_period_for_slashing: BlockNumber,

    // Amount to slash
    slash_amount: Balance,
}

#[derive(Encode, Decode, Debug, Default, Eq, PartialEq)]
pub struct UnstakingState<BlockNumber> {
    // The block where the unstaking was initiated
    started_in_block: BlockNumber,

    // Whether unstaking is in active, or conversely paused state
    // Blocks are only counted towards unstaking period when active.
    is_active: bool,

    // The number blocks which must be finalised while in the active period before the unstaking is finished
    blocks_remaining_in_active_period_for_unstaking: BlockNumber,
}

#[derive(Encode, Decode, Debug, Eq, PartialEq)]
pub enum StakedStatus<BlockNumber> {
    // Baseline staking status, nothing is happening.
    Normal,

    // Unstaking is under way
    Unstaking(UnstakingState<BlockNumber>),
}

impl<BlockNumber> Default for StakedStatus<BlockNumber> {
    fn default() -> Self {
        StakedStatus::Normal
    }
}

#[derive(Encode, Decode, Debug, Default, Eq, PartialEq)]
pub struct StakedState<BlockNumber, Balance> {
    // Total amount of funds at stake
    staked_amount: Balance,

    // All ongoing slashing process.
    // There may be some issue with BTreeMap for now in Polkadotjs,
    // consider replacing this with Vec<Slash<T>>, and remove nextSlashId from state, for now in that case,
    ongoing_slashes: BTreeMap<SlashId, Slash<BlockNumber, Balance>>,

    // Status of the staking
    staked_status: StakedStatus<BlockNumber>,
}

#[derive(Encode, Decode, Debug, Eq, PartialEq)]
pub enum StakingStatus<BlockNumber, Balance> {
    NotStaked,

    Staked(StakedState<BlockNumber, Balance>),
}

impl<BlockNumber, Balance> Default for StakingStatus<BlockNumber, Balance> {
    fn default() -> Self {
        StakingStatus::NotStaked
    }
}

#[derive(Encode, Decode, Default, Debug, Eq, PartialEq)]
pub struct Stake<BlockNumber, Balance> {
    // When role was created
    created: BlockNumber,

    // Status of any possible ongoing staking
    staking_status: StakingStatus<BlockNumber, Balance>,
}

decl_storage! {
    trait Store for Module<T: Trait> as StakePool {
        /// Maps identifiers to a stake.
        Stakes get(stakes): map StakeId => Stake<T::BlockNumber, BalanceOf<T>>;

        /// Identifier value for next stake.
        NextStakeId get(next_stake_id): StakeId = FIRST_STAKE_ID;

        /// Identifier value for next slashing.
        NextSlashId get(next_slash_id): SlashId = FIRST_SLASH_ID;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn on_finalize(now: T::BlockNumber) {
            Self::finalize_unstaking_and_slashing(now);
        }
    }
}

pub enum StakingError {
    StakeNotFound,
    SlashNotFound,
    InsufficientBalance,
    InsufficientStake,
    AlreadyStaked,
    AlreadyUnstaking,
    NotStaked,
    NotUnstaking,
    IncreasingStakeWhileUnstaking,
    DecreasingStakeWhileUnstaking,
    InsufficientBalanceInStakeFund,
    InitiatingUnstakingWhileSlashesOngoing,
}

impl<T: Trait> Module<T> {
    /// The account ID of the stake pool.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once. Is it deterministic?
    pub fn staking_fund_account_id() -> T::AccountId {
        ModuleId(T::StakePoolId::get()).into_account()
    }

    pub fn staking_fund_balance() -> BalanceOf<T> {
        T::Currency::free_balance(&Self::staking_fund_account_id())
    }

    /// Adds a new Stake which is NotStaked, created at given block, into stakes map with id NextStakeId, and increments NextStakeId.
    pub fn create_stake() -> StakeId {
        let id = Self::next_stake_id();
        <Stakes<T>>::insert(
            &id,
            Stake {
                created: <system::Module<T>>::block_number(),
                staking_status: Default::default(),
            },
        );
        NextStakeId::mutate(|id| *id += 1);
        id
    }

    /// Given that stake with id exists in stakes and is NotStaked, remove from stakes.
    pub fn remove_stake(id: &StakeId) {
        if !<Stakes<T>>::exists(id) {
            return;
        }

        match Self::stakes(id).staking_status {
            StakingStatus::NotStaked => {
                <Stakes<T>>::remove(id);
            }
            _ => (),
        }

        // do we want to return error on failure or should it be silent?
    }

    /// Provided the stake exists and is in state NotStaked and the given account has
    /// sufficient free balance to cover the given staking amount, then the amount is transferred
    /// to the module's account, and the corresponding staked_balance is set to this amount in the new Staked state.
    pub fn stake(
        id: &StakeId,
        staker_account_id: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(id);

        ensure!(
            stake.staking_status == StakingStatus::NotStaked,
            StakingError::AlreadyStaked
        );

        Self::move_funds_into_pool(staker_account_id, amount)?;

        stake.staking_status = StakingStatus::Staked(StakedState {
            staked_amount: amount,
            ongoing_slashes: BTreeMap::new(),
            staked_status: StakedStatus::Normal,
        });

        <Stakes<T>>::insert(id, stake);

        Ok(())
    }

    /// Moves funds from specified account into the module's account
    /// We don't use T::Currency::transfer() to prevent fees being incurred.
    fn move_funds_into_pool(
        source: &T::AccountId,
        value: BalanceOf<T>,
    ) -> Result<(), StakingError> {
        match T::Currency::withdraw(
            source,
            value,
            WithdrawReason::Transfer,
            ExistenceRequirement::KeepAlive,
        ) {
            Ok(negative_imbalance) => {
                // move the negative imbalance into the stake pool
                T::Currency::resolve_creating(&Self::staking_fund_account_id(), negative_imbalance);
                Ok(())
            }
            Err(_) => Err(StakingError::InsufficientBalance),
        }
    }

    /// Moves funds from the module's account into specified account.
    /// We don't use T::Currency::transfer() to prevent fees being incurred.
    fn withdraw_funds_from_pool(
        destination: &T::AccountId,
        value: BalanceOf<T>,
    ) -> Result<(), StakingError> {
        match T::Currency::withdraw(
            &Self::staking_fund_account_id(),
            value,
            WithdrawReason::Transfer,
            ExistenceRequirement::KeepAlive,
        ) {
            Ok(negative_imbalance) => {
                // move the negative imbalance into the destination account
                T::Currency::resolve_creating(destination, negative_imbalance);
                Ok(())
            }
            Err(_) => Err(StakingError::InsufficientBalanceInStakeFund),
        }
    }

    /// Provided the stake exists and is in state Staked.Normal, and the given source account covers the amount,
    /// then the amount is transferred to the module's account, and the corresponding staked_amount is increased
    /// by the amount. New value of staked_amount is returned.
    pub fn increase_stake(
        id: &StakeId,
        staker_account_id: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, StakingError> {
        ensure!(<Stakes<T>>::exists(id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(id);

        let total_staked_amount = match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => match staked_state.staked_status {
                StakedStatus::Normal => {
                    staked_state.staked_amount += amount;
                    staked_state.staked_amount
                }
                _ => return Err(StakingError::IncreasingStakeWhileUnstaking),
            },
            _ => return Err(StakingError::NotStaked),
        };

        Self::move_funds_into_pool(&staker_account_id, amount)?;

        <Stakes<T>>::insert(id, stake);

        Ok(total_staked_amount)
    }

    // !! SHOULD WE ALLOW DECREASING STAKE IF THERE ARE ACTIVE SLASHINGs ? !!
    /// Provided the stake exists and is in state Staked.Normal, and the given stake holds at least the amount,
    /// then the amount is transferred to from the module's account to the staker account, and the corresponding
    /// staked_amount is decreased by the amount. New value of staked_amount is returned.
    pub fn decrease_stake(
        id: &StakeId,
        staker_account_id: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, StakingError> {
        ensure!(<Stakes<T>>::exists(id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(id);

        let total_staked_amount = match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => match staked_state.staked_status {
                StakedStatus::Normal => {
                    if staked_state.staked_amount >= amount {
                        staked_state.staked_amount -= amount;
                        staked_state.staked_amount
                    } else {
                        return Err(StakingError::InsufficientStake);
                    }
                }
                _ => return Err(StakingError::DecreasingStakeWhileUnstaking),
            },
            _ => return Err(StakingError::NotStaked),
        };

        Self::withdraw_funds_from_pool(&staker_account_id, amount)?;

        <Stakes<T>>::insert(id, stake);

        Ok(total_staked_amount)
    }

    // Initiate a new slashing of a staked stake.
    pub fn initiate_slashing(
        stake_id: &StakeId,
        slash_amount: BalanceOf<T>,
        slash_period: T::BlockNumber,
    ) -> Result<SlashId, StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);

        // ensure!(slash_period > 0, StakingError::XXXX);

        let mut stake = Self::stakes(stake_id);

        let slash_id = match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => {
                // ensure!(staked_state.staked_amount >= slash_amount, StakingError::XXXX);

                let slash_id = NextSlashId::mutate(|next_id| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });

                staked_state.ongoing_slashes.insert(
                    slash_id,
                    Slash {
                        is_active: true,
                        blocks_remaining_in_active_period_for_slashing: slash_period,
                        slash_amount,
                        started_at_block: <system::Module<T>>::block_number(),
                    },
                );

                slash_id
            }
            _ => return Err(StakingError::NotStaked),
        };

        <Stakes<T>>::insert(stake_id, stake);

        Ok(slash_id)
    }

    // Pause an ongoing slashing
    pub fn pause_slashing(stake_id: &StakeId, slash_id: &SlashId) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => {
                match staked_state.ongoing_slashes.get_mut(slash_id) {
                    Some(ref mut slash) => {
                        if slash.is_active {
                            slash.is_active = false;
                            <Stakes<T>>::insert(stake_id, stake);
                        }
                        Ok(())
                    }
                    _ => Err(StakingError::SlashNotFound),
                }
            }
            _ => Err(StakingError::NotStaked),
        }
    }

    // Resume a currently paused ongoing slashing.
    pub fn resume_slashing(stake_id: &StakeId, slash_id: &SlashId) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => {
                match staked_state.ongoing_slashes.get_mut(slash_id) {
                    Some(ref mut slash) => {
                        if !slash.is_active {
                            slash.is_active = true;
                            <Stakes<T>>::insert(stake_id, stake);
                        }
                        Ok(())
                    }
                    _ => Err(StakingError::SlashNotFound),
                }
            }
            _ => Err(StakingError::NotStaked),
        }
    }

    // Cancel an ongoing slashing (regardless of whether its active or paused).
    pub fn cancel_slashing(stake_id: &StakeId, slash_id: &SlashId) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);

        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => {
                if staked_state.ongoing_slashes.contains_key(slash_id) {
                    staked_state.ongoing_slashes.remove(slash_id);
                    <Stakes<T>>::insert(stake_id, stake);
                    Ok(())
                } else {
                    Err(StakingError::SlashNotFound)
                }
            }
            _ => Err(StakingError::NotStaked),
        }
    }

    // Initiate unstaking of a staked stake.
    pub fn initiate_unstaking(
        stake_id: &StakeId,
        unstaking_period: T::BlockNumber,
    ) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);
        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => {
                if staked_state.ongoing_slashes.is_empty() {
                    Err(StakingError::InitiatingUnstakingWhileSlashesOngoing)
                } else {
                    match staked_state.staked_status {
                        StakedStatus::Normal => {
                            staked_state.staked_status = StakedStatus::Unstaking(UnstakingState {
                                started_in_block: <system::Module<T>>::block_number(),
                                is_active: true,
                                blocks_remaining_in_active_period_for_unstaking: unstaking_period,
                            });
                            <Stakes<T>>::insert(stake_id, stake);
                            Ok(())
                        }
                        _ => Err(StakingError::AlreadyUnstaking),
                    }
                }
            }
            _ => Err(StakingError::NotStaked),
        }
    }

    // Pause an ongoing unstaking.
    pub fn pause_unstaking(stake_id: &StakeId) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);
        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => match staked_state.staked_status {
                StakedStatus::Unstaking(ref mut unstaking_state) => {
                    if unstaking_state.is_active {
                        unstaking_state.is_active = false;
                        <Stakes<T>>::insert(stake_id, stake);
                    }
                    Ok(())
                }
                _ => Err(StakingError::NotUnstaking),
            },
            _ => Err(StakingError::NotStaked),
        }
    }

    // Continue a currently paused ongoing unstaking.
    pub fn resume_unstaking(stake_id: &StakeId) -> Result<(), StakingError> {
        ensure!(<Stakes<T>>::exists(stake_id), StakingError::StakeNotFound);
        let mut stake = Self::stakes(stake_id);

        match stake.staking_status {
            StakingStatus::Staked(ref mut staked_state) => match staked_state.staked_status {
                StakedStatus::Unstaking(ref mut unstaking_state) => {
                    if !unstaking_state.is_active {
                        unstaking_state.is_active = true;
                        <Stakes<T>>::insert(stake_id, stake);
                    }
                    Ok(())
                }
                _ => Err(StakingError::NotUnstaking),
            },
            _ => Err(StakingError::NotStaked),
        }
    }

    /*
      Handle timers for finalizing unstaking and slashing.
      Finalised unstaking results in the staked_balance in the given stake to be transferred.
      Finalised slashing results in the staked_balance in the given stake being correspondingly reduced.
    */
    fn finalize_unstaking_and_slashing(now: T::BlockNumber) {}
}
