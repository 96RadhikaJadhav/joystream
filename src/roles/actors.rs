#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use parity_codec::Codec;
use parity_codec_derive::{Encode, Decode};
use srml_support::{StorageMap, StorageValue, dispatch, decl_module, decl_storage, decl_event, ensure, Parameter};
use srml_support::traits::{Currency, EnsureAccountLiquid};
use runtime_primitives::traits::{Zero, Bounded, SimpleArithmetic, As, Member, MaybeSerializeDebug};
use system::{self, ensure_signed};
use crate::governance::{GovernanceCurrency, BalanceOf };
use crate::membership::registry;

use crate::traits::{Members, Roles};

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq, Debug)]
pub enum Role {
    Storage,
}

#[derive(Encode, Decode, Clone)]
pub struct RoleParameters<T: Trait> {
    // minium balance required to stake to enter a role
    min_stake: BalanceOf<T>,

    // the maximum number of spots available to fill for a role
    max_actors: u32,

    // minimum actors to maintain - if role is unstaking
    // and remaining actors would be less that this value - prevent or punish for unstaking
    min_actors: u32,

    // fixed amount of tokens paid to actors' primary account
    reward_per_block: BalanceOf<T>,

    // payouts are made at this block interval
    reward_period: T::BlockNumber,

    // how long tokens remain locked for after unstaking
    unbonding_period: T::BlockNumber,

    // minimum amount of time before being able to unstake
    bonding_time: T::BlockNumber,

    // minimum period required to be in service. unbonding before this time is highly penalized
    min_service_period: T::BlockNumber,

    // "startup" time allowed for roles that need to sync their infrastructure
    // with other providers before they are considered in service and punishable for
    // not delivering required level of service.
    startup_grace_period: T::BlockNumber,

    // entry_request_fee: BalanceOf<T>,
}

#[derive(Encode, Decode, Clone)]
pub struct Actor<T: Trait> {
    member_id: <T::Members as Members<T>>::Id,
    role: Role,
    account: T::AccountId,
    joined_at: T::BlockNumber,
    // startup_grace_period_ends_at: T::BlockNumber,
}

pub trait Trait: system::Trait + GovernanceCurrency {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Members: Members<Self>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Actors {
        /// requirements to enter and maintain status in roles
        Parameters get(parameters) : map Role => Option<RoleParameters<T>>;

        /// the roles members can enter into
        AvailableRoles get(available_roles) : Vec<Role>; // = vec![Role::Storage];

        /// actor accounts mapped to their actor
        Actors get(actors) : map T::AccountId => Option<Actor<T>>;

        /// actor accounts associated with a role
        AccountsByRole get(accounts_by_role) : map Role => Vec<T::AccountId>;

        /// actor accounts associated with a member id
        AccountsByMember get(accounts_by_member) : map <T::Members as Members<T>>::Id => Vec<T::AccountId>;

        /// tokens locked until given block number
        Bondage get(bondage) : map T::AccountId => T::BlockNumber;

        /// First step before enter a role is registering intent with a new account/key.
        /// This is done by sending a role_entry_request() from the new account.
        /// The member must then send a stake() transaction to approve the request and enter the desired role.
        /// This list is cleared every N blocks.. the account making the request will be bonded and must have
        /// sufficient balance to cover the minimum stake for the role.
        /// Bonding only occurs after successful entry into a role.
        RoleEntryRequests get(role_entry_requests) : map T::AccountId => Option<(<T::Members as Members<T>>::Id, Role)>;
    }
}

decl_event! {
    pub enum Event<T> where
      <T as system::Trait>::AccountId {
        Staked(AccountId, Role),
    }
}

impl<T: Trait> Module<T> {
    fn role_is_available(role: Role) -> bool {
        Self::available_roles().into_iter().any(|r| role == r)
    }

    fn ensure_actor(role_key: &T::AccountId) -> Result<Actor<T>, &'static str> {
        Self::actors(role_key).ok_or("not role key")
    }

    fn ensure_actor_is_member(role_key: &T::AccountId, member_id: <T::Members as Members<T>>::Id)
        -> Result<Actor<T>, &'static str>
    {
        let actor = Self::ensure_actor(role_key)?;
        if actor.member_id == member_id {
            Ok(actor)
        } else {
            Err("actor not owned by member")
        }
    }

    fn ensure_role_parameters(role: Role) -> Result<RoleParameters<T>, &'static str> {
        Self::parameters(role).ok_or("no parameters for role")
    }
}

impl<T: Trait> Roles<T> for Module<T> {
    fn is_role_account(account_id: &T::AccountId) -> bool {
        <Actors<T>>::exists(account_id) || <Bondage<T>>::exists(account_id) || <RoleEntryRequests<T>>::exists(account_id)
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn on_finalize(now: T::BlockNumber) {
            // clear <RoleEntryRequests<T>>::kill() every N blocks

            // payout rewards to actors

            // clear unbonded accounts
        }

        pub fn role_entry_request(origin, role: Role, member_id: <T::Members as Members<T>>::Id) {
            let sender = ensure_signed(origin)?;

            ensure!(T::Members::lookup_member_id(&sender).is_err(), "account is a member");
            ensure!(!Self::is_role_account(&sender), "account already used");

            ensure!(Self::role_is_available(role), "inactive role");

            let role_parameters = Self::ensure_role_parameters(role)?;

            <RoleEntryRequests<T>>::insert(&sender, (member_id, role));
        }

        /// Member activating entry request
        pub fn stake(origin, role: Role, actor_account: T::AccountId) {
            let sender = ensure_signed(origin)?;
            let member_id = T::Members::lookup_member_id(&sender)?;

            ensure!(<RoleEntryRequests<T>>::exists(&actor_account), "no role entry request matches");
            if let Some(entry_request) = Self::role_entry_requests(&actor_account) {
                ensure!(entry_request.0 == member_id, "role entry mismatch - member id");
                ensure!(entry_request.1 == role, "role entry mismatch - role");
            }

            // make sure role is still available
            ensure!(Self::role_is_available(role), "");
            let role_parameters = Self::ensure_role_parameters(role)?;

            let accounts_in_role = Self::accounts_by_role(role);

            // ensure there is an empty slot for the role
            ensure!(accounts_in_role.len() < role_parameters.max_actors as usize, "role slots full");

            // ensure the actor account has enough balance
            ensure!(T::Currency::free_balance(&actor_account) >= role_parameters.min_stake, "");

            <AccountsByRole<T>>::mutate(role, |accounts| accounts.push(actor_account.clone()));
            <AccountsByMember<T>>::mutate(&member_id, |accounts| accounts.push(actor_account.clone()));
            <Bondage<T>>::insert(&actor_account, T::BlockNumber::max_value());
            <Actors<T>>::insert(&actor_account, Actor {
                member_id,
                account: actor_account.clone(),
                role,
                joined_at: <system::Module<T>>::block_number()
            });
            <RoleEntryRequests<T>>::remove(&actor_account);

            Self::deposit_event(RawEvent::Staked(actor_account, role));
        }

        pub fn unstake(origin, actor_account: T::AccountId) {
            let sender = ensure_signed(origin)?;
            let member_id = T::Members::lookup_member_id(&sender)?;

            let actor = Self::ensure_actor_is_member(&actor_account, member_id)?;

            let role_parameters = Self::ensure_role_parameters(actor.role)?;

            // simple unstaking ...only applying unbonding period
            let accounts: Vec<T::AccountId> = Self::accounts_by_role(actor.role)
                .into_iter()
                .filter(|account| !(*account == actor.account))
                .collect();
            <AccountsByRole<T>>::insert(actor.role, accounts);

            let accounts: Vec<T::AccountId> = Self::accounts_by_member(&member_id)
                .into_iter()
                .filter(|account| !(*account == actor.account))
                .collect();
            <AccountsByMember<T>>::insert(&member_id, accounts);

            <Bondage<T>>::insert(&actor_account, <system::Module<T>>::block_number() + role_parameters.unbonding_period);

            <Actors<T>>::remove(&actor_account);
        }

        // pub fn set_role_parameters(role: Role, params: RoleParameters) {}
        // pub fn set_available_roles(Vec<Role>) {}
    }
}

impl<T: Trait> EnsureAccountLiquid<T::AccountId> for Module<T> {
	fn ensure_account_liquid(who: &T::AccountId) -> dispatch::Result {
		if Self::bondage(who) <= <system::Module<T>>::block_number() {
			Ok(())
		} else {
			Err("cannot transfer illiquid funds")
		}
	}
}