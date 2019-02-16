#![cfg_attr(not(feature = "std"), no_std)]
use rstd::prelude::*;
use srml_support::{StorageValue, StorageMap, dispatch::Result, decl_module, decl_event, decl_storage, ensure};
use system::{self, ensure_signed};
pub use super::{ GovernanceCurrency, BalanceOf };

use super::{council, election::{self, TriggerElection}};

pub trait Trait: system::Trait + council::Trait + election::Trait + GovernanceCurrency {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type TriggerElection: election::TriggerElection<election::Seats<Self::AccountId, BalanceOf<Self>>, election::ElectionParameters<Self::BlockNumber, BalanceOf<Self>>>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Root {
        // Electin Parameters to be used on the next election
        NextElectionParameters get(next_election_parameters) config(): election::ElectionParameters<T::BlockNumber, BalanceOf<T>>;

        // Flag for wether to automatically start an election after a council term ends
        AutoStartElections get(auto_start_elections) : bool = true;
    }
}

/// Event for this module.
decl_event!(
    pub enum Event<T> where <T as system::Trait>::BlockNumber {
        // TODO add more useful info to events?
        ElectionStarted(),
        CouncilTermEnded(),
        Dummy(BlockNumber),
    }
);

impl<T: Trait> Module<T> {
    // Nothing yet
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        fn set_next_election_parameters (params: election::ElectionParameters<T::BlockNumber, BalanceOf<T>>) {
            <NextElectionParameters<T>>::put(params);
        }

        fn set_auto_start_elections (flag: bool) {
            <AutoStartElections<T>>::put(flag);
        }

        fn start_election() {
            let current_council = <council::Module<T>>::active_council();

            let params = Self::next_election_parameters();

            if T::TriggerElection::trigger_election(current_council, params).is_ok() {
                Self::deposit_event(RawEvent::ElectionStarted());
            }
        }
    }
}

impl<T: Trait> council::CouncilTermEnded for Module<T> {
    fn council_term_ended() {
        Self::deposit_event(RawEvent::CouncilTermEnded());

        if Self::auto_start_elections() && !<election::Module<T>>::is_election_running() {
            Self::start_election();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::mock::*;
    use runtime_io::with_externalities;
    use srml_support::*;

    #[test]
    fn election_is_triggerred_when_council_term_ends() {
        with_externalities(&mut initial_test_ext(), || {
            System::set_block_number(1);

            assert!(Council::is_term_ended(1));
            assert!(Election::stage().is_none());

            <Governance as council::CouncilTermEnded>::council_term_ended();

            assert!(Election::stage().is_some());
        });
    }
}