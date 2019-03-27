#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use parity_codec::Codec;
use parity_codec_derive::{Encode, Decode};
use srml_support::{StorageMap, StorageValue, decl_module, decl_storage, decl_event, ensure, Parameter};
use runtime_primitives::traits::{SimpleArithmetic, As, Member, MaybeSerializeDebug, MaybeDebug};
use system::{self, ensure_root};
use crate::traits;

pub trait Trait: system::Trait + MaybeDebug {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type DataObjectTypeId: Parameter + Member + SimpleArithmetic + Codec + Default + Copy
        + As<usize> + As<u64> + MaybeSerializeDebug + PartialEq;
}

static MSG_DO_TYPE_NOT_FOUND: &str = "Data Object Type with the given ID not found!";

const DEFAULT_FIRST_DATA_OBJECT_TYPE_ID: u64 = 1;

#[derive(Clone, Encode, Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DataObjectType {
    pub description: Vec<u8>,
    pub active: bool,

    // TODO in future releases
    // - maximum size
    // - replication factor
    // - storage tranches (empty is ok)
}

decl_storage! {
    trait Store for Module<T: Trait> as DataObjectTypeRegistry {
        // Start at this value
        pub FirstDataObjectTypeId get(first_data_object_type_id) config(first_data_object_type_id): T::DataObjectTypeId = T::DataObjectTypeId::sa(DEFAULT_FIRST_DATA_OBJECT_TYPE_ID);

        // Increment
        pub NextDataObjectTypeId get(next_data_object_type_id) build(|config: &GenesisConfig<T>| config.first_data_object_type_id): T::DataObjectTypeId = T::DataObjectTypeId::sa(DEFAULT_FIRST_DATA_OBJECT_TYPE_ID);

        // Mapping of Data object types
        pub DataObjectTypeMap get(data_object_type): map T::DataObjectTypeId => Option<DataObjectType>;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as Trait>::DataObjectTypeId {
        DataObjectTypeRegistered(DataObjectTypeId),
        DataObjectTypeUpdated(DataObjectTypeId),
    }
}



impl<T: Trait> traits::IsActiveDataObjectType<T> for Module<T> {
    fn is_active_data_object_type(which: &T::DataObjectTypeId) -> bool {
        match Self::ensure_data_object_type(*which) {
            Ok(do_type) => do_type.active,
            Err(_err) => false
        }
    }
}


decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn register_data_object_type(origin, data_object_type: DataObjectType) {
            ensure_root(origin)?;

            let new_do_type_id = Self::next_data_object_type_id();
            let do_type: DataObjectType = DataObjectType {
                description: data_object_type.description.clone(),
                active: data_object_type.active,
            };

            <DataObjectTypeMap<T>>::insert(new_do_type_id, do_type);
            <NextDataObjectTypeId<T>>::mutate(|n| { *n += T::DataObjectTypeId::sa(1); });

            Self::deposit_event(RawEvent::DataObjectTypeRegistered(new_do_type_id));
        }

        pub fn update_data_object_type(origin, id: T::DataObjectTypeId, data_object_type: DataObjectType) {
            ensure_root(origin)?;
            let mut do_type = Self::ensure_data_object_type(id)?;

            do_type.description = data_object_type.description.clone();
            do_type.active = data_object_type.active;

            <DataObjectTypeMap<T>>::insert(id, do_type);

            Self::deposit_event(RawEvent::DataObjectTypeUpdated(id));
        }

        // Activate and deactivate functions as separate functions, because
        // toggling DO types is likely a more common operation than updating
        // other aspects.
        pub fn activate_data_object_type(origin, id: T::DataObjectTypeId) {
            ensure_root(origin)?;
            let mut do_type = Self::ensure_data_object_type(id)?;

            do_type.active = true;

            <DataObjectTypeMap<T>>::insert(id, do_type);

            Self::deposit_event(RawEvent::DataObjectTypeUpdated(id));
        }

        pub fn deactivate_data_object_type(origin, id: T::DataObjectTypeId) {
            ensure_root(origin)?;
            let mut do_type = Self::ensure_data_object_type(id)?;

            do_type.active = false;

            <DataObjectTypeMap<T>>::insert(id, do_type);

            Self::deposit_event(RawEvent::DataObjectTypeUpdated(id));
        }

    }
}

impl <T: Trait> Module<T> {
    fn ensure_data_object_type(id: T::DataObjectTypeId) -> Result<DataObjectType, &'static str> {
        return Self::data_object_type(&id).ok_or(MSG_DO_TYPE_NOT_FOUND);
    }
}
