#![cfg_attr(not(feature = "std"), no_std)]

extern crate sr_std;
#[cfg(test)]
extern crate sr_io;
#[cfg(test)]
extern crate substrate_primitives;
extern crate sr_primitives;
#[cfg(feature = "std")]
extern crate parity_codec as codec;
extern crate srml_system as system;

pub mod election;
pub mod council;
pub mod governance;

mod transferable_stake;
mod sealed_vote;

pub use self::governance::{Trait, Module, RawEvent, Event};

#[cfg(test)]
pub mod tests {
    pub use super::*;

    use self::sr_io::with_externalities;
    use self::substrate_primitives::{H256, Blake2Hasher};
    use self::sr_primitives::{
        BuildStorage, traits::BlakeTwo256, traits::IdentityLookup, testing::{Digest, DigestItem, Header}
    };


    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
        type Lookup = IdentityLookup<u64>;
    }
    impl council::Trait for Test {
        type Event = ();
    }
    impl election::Trait for Test {
        type Event = ();
    }
    impl balances::Trait for Test {
        type Event = ();

        /// The balance of an account.
        type Balance = u32;

        /// A function which is invoked when the free-balance has fallen below the existential deposit and
        /// has been reduced to zero.
        ///
        /// Gives a chance to clean up resources associated with the given account.
        type OnFreeBalanceZero = ();

        /// Handler for when a new account is created.
        type OnNewAccount = ();

        /// A function that returns true iff a given account can transfer its funds to another account.
        type EnsureAccountLiquid = ();
    }
    impl governance::Trait for Test {
        type Event = ();
    }

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn  initial_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
        /*
        t.extend(governance::GenesisConfig::<Test>{
            //items
        }.build_storage().unwrap().0);
        */
        runtime_io::TestExternalities::new(t)
    }

    #[test]
    fn election_starts() {
        with_externalities(&mut initial_test_ext(), || {
            assert_eq!(Election::round(), 0);
            System::set_block_number(1);

            assert!(Election::start_election().is_ok());

            // election round is bumped
            assert_eq!(Election::round(), 1);

            // we enter the announcing stage for a specified period
            let expected_period = election::Period {
                starts: 1,
                ends: 1 + election::ANNOUNCING_PERIOD
            };

            if let Some(election_stage) = Election::stage() {
                match election_stage {
                    election::Stage::Announcing(period) => assert_eq!(period, expected_period),
                    _ => assert!(false)
                }
            } else {
                assert!(false);
            }
        });
    }

    pub type Governance = Module<Test>;
    pub type Election = election::Module<Test>;
    pub type Council = council::Module<Test>;
	pub type System = system::Module<Test>;
	pub type Balances = balances::Module<Test>;
}