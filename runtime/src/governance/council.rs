#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std;
#[cfg(test)]
extern crate sr_io;
#[cfg(test)]
extern crate substrate_primitives;
extern crate sr_primitives;
#[cfg(feature = "std")]
extern crate parity_codec as codec;
//extern crate srml_system as system;
use srml_support::dispatch::Vec;

use srml_support::{StorageValue, dispatch::Result};
use runtime_primitives::traits::{Hash, As};
use {balances, system::{self, ensure_signed}};

pub trait Trait: system::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Clone, Encode, Decode)]
pub struct Seat<Id, Stake> {
    pub member: Id,
    pub stake: Stake,
    pub backers: Vec<Backer<Id, Stake>>,
}

#[derive(Copy, Clone, Encode, Decode)]
pub struct Backer<Id, Stake> {
    pub member: Id,
    pub stake: Stake,
}

pub type Council<AccountId, Balance> = Vec<Seat<AccountId, Balance>>;

decl_storage! {
    trait Store for Module<T: Trait> as CouncilInSession {
        // Initial state - council is empty and resigned, which will trigger
        // and election in next block
        ActiveCouncil get(council): Option<Council<T::AccountId, T::Balance>>;

        TermEnds get(term_ends): T::BlockNumber = T::BlockNumber::sa(0);
    }
}

/// Event for this module.
decl_event!(
	pub enum Event<T> where <T as system::Trait>::BlockNumber {
        CouncilResigned(BlockNumber),
        CouncilTermEnded(BlockNumber),
	}
);

impl<T: Trait> Module<T> {
    pub fn set_council() {

    }

    pub fn term_ended(n: T::BlockNumber) -> bool {
        n >= Self::term_ends()
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
    }
}