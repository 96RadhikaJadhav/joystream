use frame_support::traits::{
    Currency, LockIdentifier, LockableCurrency, OnFinalize, OnInitialize, WithdrawReasons,
};
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError, Perbill,
};
use system;

use crate::{BalanceOfCurrency, DefaultInstance, Module, StakingHandler, Trait};
use common::currency::GovernanceCurrency;
use frame_support::dispatch::DispatchResult;

impl_outer_origin! {
    pub enum Origin for Test {}
}

mod working_team {
    pub use crate::Event;
}

mod membership_mod {
    pub use membership::Event;
}

impl_outer_event! {
    pub enum TestEvent for Test {
        balances<T>,
        crate DefaultInstance <T>,
        membership_mod<T>,
        system<T>,
    }
}

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumPeriod: u64 = 5;
    pub const StakePoolId: [u8; 8] = *b"joystake";
    pub const ExistentialDeposit: u32 = 0;
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 - remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Test;

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
}

impl common::currency::GovernanceCurrency for Test {
    type Currency = Balances;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = TestEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

impl membership::Trait for Test {
    type Event = TestEvent;
    type MemberId = u64;
    type PaidTermId = u64;
    type SubscriptionId = u64;
    type ActorId = u64;
}

pub type Membership = membership::Module<Test>;
pub type Balances = balances::Module<Test>;
pub type System = system::Module<Test>;

parameter_types! {
    pub const RewardPeriod: u32 = 2;
    pub const MaxWorkerNumberLimit: u32 = 3;
    pub const MinUnstakingPeriodLimit: u64 = 3;
    pub const LockId: [u8; 8] = [1; 8];
}

impl Trait for Test {
    type OpeningId = u64;
    type ApplicationId = u64;
    type Event = TestEvent;
    type MaxWorkerNumberLimit = MaxWorkerNumberLimit;
    type StakingHandler = Test;
    type MemberOriginValidator = ();
    type MinUnstakingPeriodLimit = MinUnstakingPeriodLimit;
    type RewardPeriod = RewardPeriod;
}

pub const ACTOR_ORIGIN_ERROR: &'static str = "Invalid membership";

impl common::origin::ActorOriginValidator<Origin, u64, u64> for () {
    fn ensure_actor_origin(origin: Origin, member_id: u64) -> Result<u64, &'static str> {
        let signed_account_id = system::ensure_signed(origin)?;

        if member_id > 10 {
            return Err(ACTOR_ORIGIN_ERROR);
        }

        Ok(signed_account_id)
    }
}

pub type TestWorkingTeam = Module<Test, DefaultInstance>;

pub const STAKING_ACCOUNT_ID_FOR_FAILED_VALIDITY_CHECK: u64 = 111;
pub const STAKING_ACCOUNT_ID_FOR_FAILED_AMOUNT_CHECK: u64 = 222;
pub const STAKING_ACCOUNT_ID_FOR_CONFLICTING_STAKES: u64 = 333;
pub const STAKING_ACCOUNT_ID_FOR_ZERO_STAKE: u64 = 444;
pub const LOCK_ID: LockIdentifier = [1; 8];

impl StakingHandler<Test> for Test {
    fn lock(account_id: &<Test as system::Trait>::AccountId, amount: BalanceOfCurrency<Test>) {
        <Test as GovernanceCurrency>::Currency::set_lock(
            LOCK_ID,
            &account_id,
            amount,
            WithdrawReasons::all(),
        )
    }

    fn unlock(account_id: &<Test as system::Trait>::AccountId) {
        <Test as GovernanceCurrency>::Currency::remove_lock(LOCK_ID, &account_id);
    }

    fn slash(
        account_id: &<Test as system::Trait>::AccountId,
        amount: Option<BalanceOfCurrency<Test>>,
    ) -> BalanceOfCurrency<Test> {
        let locks = Balances::locks(&account_id);

        let existing_lock = locks.iter().find(|lock| lock.id == LOCK_ID);

        let mut actually_slashed_balance = Default::default();
        if let Some(existing_lock) = existing_lock {
            Self::unlock(&account_id);

            let mut slashable_amount = existing_lock.amount;
            if let Some(amount) = amount {
                if existing_lock.amount > amount {
                    let new_amount = existing_lock.amount - amount;
                    Self::lock(&account_id, new_amount);

                    slashable_amount = amount;
                }
            }

            let _ = Balances::slash(&account_id, slashable_amount);

            actually_slashed_balance = slashable_amount
        }

        actually_slashed_balance
    }

    fn decrease_stake(
        account_id: &<Test as system::Trait>::AccountId,
        amount: BalanceOfCurrency<Test>,
    ) {
        Self::unlock(account_id);
        Self::lock(account_id, amount);
    }

    fn increase_stake(
        account_id: &<Test as system::Trait>::AccountId,
        amount: BalanceOfCurrency<Test>,
    ) -> DispatchResult {
        if !Self::is_enough_balance_for_stake(account_id, amount) {
            return Err(DispatchError::Other("External check failed"));
        }

        Self::unlock(account_id);
        Self::lock(account_id, amount);

        Ok(())
    }

    fn is_member_staking_account(_member_id: &u64, account_id: &u64) -> bool {
        if *account_id == STAKING_ACCOUNT_ID_FOR_FAILED_VALIDITY_CHECK {
            return false;
        }

        true
    }

    fn is_account_free_of_conflicting_stakes(account_id: &u64) -> bool {
        if *account_id == STAKING_ACCOUNT_ID_FOR_CONFLICTING_STAKES {
            return false;
        }

        true
    }

    fn is_enough_balance_for_stake(account_id: &u64, amount: u64) -> bool {
        if *account_id == STAKING_ACCOUNT_ID_FOR_FAILED_AMOUNT_CHECK || amount > 1000 {
            return false;
        }

        true
    }

    fn current_stake(account_id: &u64) -> u64 {
        if *account_id == STAKING_ACCOUNT_ID_FOR_ZERO_STAKE {
            return 0;
        }

        100 // random non-zero value
    }
}

pub fn build_test_externalities() -> sp_io::TestExternalities {
    let t = system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    t.into()
}

// Recommendation from Parity on testing on_finalize
// https://substrate.dev/docs/en/next/development/module/tests
pub fn run_to_block(n: u64) {
    while System::block_number() < n {
        <System as OnFinalize<u64>>::on_finalize(System::block_number());
        <TestWorkingTeam as OnFinalize<u64>>::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        <System as OnInitialize<u64>>::on_initialize(System::block_number());
        <TestWorkingTeam as OnInitialize<u64>>::on_initialize(System::block_number());
    }
}
