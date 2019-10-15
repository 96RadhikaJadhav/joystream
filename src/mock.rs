#![cfg(test)]

use crate::*;
use crate::{Module, Trait};

use primitives::{Blake2Hasher, H256};
use runtime_io::with_externalities;
use runtime_primitives::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use srml_support::{assert_err, assert_ok, impl_outer_origin, parameter_types};
use versioned_store::InputValidationLengthConstraint;

impl_outer_origin! {
    pub enum Origin for Runtime {}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: u32 = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::one();
    pub const MinimumPeriod: u64 = 5;
}

impl system::Trait for Runtime {
    type Origin = Origin;
    type Index = u64;
    type BlockNumber = u64;
    type Call = ();
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type WeightMultiplierUpdate = ();
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
}

impl timestamp::Trait for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

impl versioned_store::Trait for Runtime {
    type Event = ();
}

impl Trait for Runtime {
    type GroupId = u64;
    type GroupMembershipChecker = ();
    type CreateClassPermissionsChecker = ();
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.

fn default_versioned_store_genesis_config() -> versioned_store::GenesisConfig {
    versioned_store::GenesisConfig {
        class_by_id: vec![],
        entity_by_id: vec![],
        next_class_id: 1,
        next_entity_id: 1,
        property_name_constraint: InputValidationLengthConstraint {
            min: 1,
            max_min_diff: 49,
        },
        property_description_constraint: InputValidationLengthConstraint {
            min: 0,
            max_min_diff: 500,
        },
        class_name_constraint: InputValidationLengthConstraint {
            min: 1,
            max_min_diff: 49,
        },
        class_description_constraint: InputValidationLengthConstraint {
            min: 0,
            max_min_diff: 500,
        },
    }
}

fn build_test_externalities(
    config: versioned_store::GenesisConfig,
) -> runtime_io::TestExternalities<Blake2Hasher> {
    let mut t = system::GenesisConfig::default()
        .build_storage::<Runtime>()
        .unwrap();

    config.assimilate_storage(&mut t).unwrap();

    t.into()
}

pub fn with_test_externalities<R, F: FnOnce() -> R>(f: F) -> R {
    let versioned_store_config = default_versioned_store_genesis_config();
    with_externalities(&mut build_test_externalities(versioned_store_config), f)
}

// pub type System = system::Module;

/// Export module on a test runtime
pub type Permissions = Module<Runtime>;
