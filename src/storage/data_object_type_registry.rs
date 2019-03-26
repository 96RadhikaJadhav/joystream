#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use parity_codec::Codec;
use parity_codec_derive::{Encode, Decode};
use srml_support::{StorageMap, StorageValue, decl_module, decl_storage, decl_event, ensure, Parameter};
use runtime_primitives::traits::{SimpleArithmetic, As, Member, MaybeSerializeDebug, MaybeDebug};
use system::{self, ensure_root};
use crate::traits;

pub trait Trait: system::Trait + MaybeDebug
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type DataObjectTypeID: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
        + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;
}


static MSG_REQUIRE_NEW_DO_TYPE: &str = "New Data Object Type required; the provided one seems to be in use already!";
static MSG_DO_TYPE_NOT_FOUND: &str = "Data Object Type with the given ID not found!";
static MSG_REQUIRE_DO_TYPE_ID: &str = "Can only update Data Object Types that are already registered (with an ID)!";

const DEFAULT_FIRST_DATA_OBJECT_TYPE_ID: u64 = 1;

#[derive(Clone, Encode, Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DataObjectType<T: Trait>
{
    // If the OT is registered, an ID must exist, otherwise it's a new OT.
    pub id: Option<T::DataObjectTypeID>,
    pub description: Vec<u8>,
    pub active: bool,

    // TODO in future releases
    // - replication factor
    // - storage tranches (empty is ok)
}

decl_storage! {
    trait Store for Module<T: Trait> as DataObjectTypeRegistry
    {
        // Start at this value
        pub FirstDataObjectTypeID get(first_data_object_type_id) config(first_data_object_type_id): T::DataObjectTypeID = T::DataObjectTypeID::sa(DEFAULT_FIRST_DATA_OBJECT_TYPE_ID);

        // Increment
        pub NextDataObjectTypeID get(next_data_object_type_id) build(|config: &GenesisConfig<T>| config.first_data_object_type_id): T::DataObjectTypeID = T::DataObjectTypeID::sa(DEFAULT_FIRST_DATA_OBJECT_TYPE_ID);

        // Mapping of Data object types
        pub DataObjectTypeMap get(data_object_type): map T::DataObjectTypeID => Option<DataObjectType<T>>;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as Trait>::DataObjectTypeID
    {
        DataObjectTypeAdded(DataObjectTypeID),
        DataObjectTypeUpdated(DataObjectTypeID),
    }
}



impl<T: Trait> traits::IsActiveDataObjectType<T> for Module<T>
{
    fn is_active_data_object_type(which: &T::DataObjectTypeID) -> bool
    {
        match Self::ensure_data_object_type(*which)
        {
            Ok(do_type) => do_type.active,
            Err(_err) => false
        }
    }
}


decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
    {
        fn deposit_event<T>() = default;

        pub fn register_data_object_type(origin, data_object_type: DataObjectType<T>)
        {
            ensure_root(origin)?;
            ensure!(data_object_type.id.is_none(), MSG_REQUIRE_NEW_DO_TYPE);

            let new_do_type_id = Self::next_data_object_type_id();
            let do_type: DataObjectType<T> = DataObjectType {
                id: Some(new_do_type_id),
                description: data_object_type.description.clone(),
                active: data_object_type.active,
            };

            <DataObjectTypeMap<T>>::insert(new_do_type_id, do_type);
            <NextDataObjectTypeID<T>>::mutate(|n| { *n += T::DataObjectTypeID::sa(1); });

            Self::deposit_event(RawEvent::DataObjectTypeAdded(new_do_type_id));
        }

        pub fn update_data_object_type(origin, data_object_type: DataObjectType<T>)
        {
            ensure_root(origin)?;
            ensure!(data_object_type.id.is_some(), MSG_REQUIRE_DO_TYPE_ID);

            let id = data_object_type.id.unwrap();
            let mut do_type = Self::ensure_data_object_type(id)?;

            do_type.description = data_object_type.description.clone();
            do_type.active = data_object_type.active;

            <DataObjectTypeMap<T>>::insert(id, do_type);

            Self::deposit_event(RawEvent::DataObjectTypeUpdated(id));
        }

        pub fn activate_data_object_type(origin, id: T::DataObjectTypeID, active: bool)
        {
            ensure_root(origin)?;
            let mut do_type = Self::ensure_data_object_type(id)?;

            do_type.active = active;

            <DataObjectTypeMap<T>>::insert(id, do_type);

            Self::deposit_event(RawEvent::DataObjectTypeUpdated(id));
        }
    }
}

impl <T: Trait> Module<T>
{
    fn ensure_data_object_type(id: T::DataObjectTypeID) -> Result<DataObjectType<T>, &'static str>
    {
        return Self::data_object_type(&id).ok_or(MSG_DO_TYPE_NOT_FOUND);
    }
}
