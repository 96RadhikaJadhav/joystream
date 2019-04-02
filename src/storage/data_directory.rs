#![cfg_attr(not(feature = "std"), no_std)]

use crate::storage::data_object_type_registry::Trait as DOTRTrait;
use crate::traits::{ContentIdExists, IsActiveDataObjectType, Members};
use parity_codec::Codec;
use parity_codec_derive::{Decode, Encode};
use rstd::prelude::*;
use runtime_primitives::traits::{MaybeDebug, MaybeSerializeDebug, Member};
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch, ensure, Parameter, StorageMap,
};
use system::{self, ensure_signed};

pub trait Trait: timestamp::Trait + system::Trait + DOTRTrait + MaybeDebug {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type ContentId: Parameter + Member + Codec + Default + Clone + MaybeSerializeDebug + PartialEq;

    type Members: Members<Self>;
    type IsActiveDataObjectType: IsActiveDataObjectType<Self>;
}

static MSG_DUPLICATE_CID: &str = "Content with this ID already exists!";
static MSG_CID_NOT_FOUND: &str = "Content with this ID not found!";
static MSG_LIAISON_REQUIRED: &str = "Only the liaison for the content may modify its status!";
static MSG_CREATOR_MUST_BE_MEMBER: &str = "Only active members may create content!";
static MSG_DO_TYPE_MUST_BE_ACTIVE: &str =
    "Cannot create content for inactive or missing data object type!";

#[derive(Clone, Encode, Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum LiaisonJudgement {
    Pending,
    Rejected,
    Accepted,
}

impl Default for LiaisonJudgement {
    fn default() -> Self {
        LiaisonJudgement::Pending
    }
}

#[derive(Clone, Encode, Decode, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DataObject<T: Trait> {
    pub data_object_type: <T as DOTRTrait>::DataObjectTypeId,
    pub size: u64,
    pub added_at_block: T::BlockNumber,
    pub added_at_time: T::Moment,
    pub owner: T::AccountId,
    pub liaison: T::AccountId,
    pub liaison_judgement: LiaisonJudgement,
}

decl_storage! {
    trait Store for Module<T: Trait> as DataDirectory {
        // Mapping of Content ID to Data Object
        pub Contents get(contents): map T::ContentId => Option<DataObject<T>>;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as Trait>::ContentId,
        <T as system::Trait>::AccountId
    {
        // The account is the Liaison that was selected
        ContentAdded(ContentId, AccountId),

        // The account is the liaison again - only they can reject or accept
        ContentAccepted(ContentId, AccountId),
        ContentRejected(ContentId, AccountId),
    }
}

impl<T: Trait> ContentIdExists<T> for Module<T> {
    fn has_content(which: &T::ContentId) -> bool {
        Self::contents(which.clone()).is_some()
    }

    fn get_data_object(which: &T::ContentId) -> Result<DataObject<T>, &'static str> {
        match Self::contents(which.clone()) {
            None => Err(MSG_CID_NOT_FOUND),
            Some(data) => Ok(data),
        }
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        pub fn add_content(origin, data_object_type_id: <T as DOTRTrait>::DataObjectTypeId,
                           id: T::ContentId, size: u64) {
            // Origin has to be a member
            let who = ensure_signed(origin)?;
            ensure!(T::Members::is_active_member(&who), MSG_CREATOR_MUST_BE_MEMBER);

            // Data object type has to be active
            ensure!(T::IsActiveDataObjectType::is_active_data_object_type(&data_object_type_id), MSG_DO_TYPE_MUST_BE_ACTIVE);

            // We essentially accept the content ID and size at face value. All we
            // need to know is that it doesn't yet exist.
            let found = Self::contents(&id);
            ensure!(found.is_none(), MSG_DUPLICATE_CID);

            // The liaison is something we need to take from staked roles. The idea
            // is to select the liaison, for now randomly.
            // FIXME without that module, we're currently hardcoding it, to the
            // origin, which is wrong on many levels.
            let liaison = who.clone();

            // Let's create the entry then
            let data: DataObject<T> = DataObject {
                data_object_type: data_object_type_id,
                size: size,
                added_at_block: <system::Module<T>>::block_number(),
                added_at_time: <timestamp::Module<T>>::now(),
                owner: who,
                liaison: liaison.clone(),
                liaison_judgement: LiaisonJudgement::Pending,
            };

            // If we've constructed the data, we can store it and send an event.
            <Contents<T>>::insert(id.clone(), data);
            Self::deposit_event(RawEvent::ContentAdded(id, liaison));
        }

        // The LiaisonJudgement can be updated, but only by the liaison.
        fn accept_content(origin, id: T::ContentId) {
            let who = ensure_signed(origin)?;
            Self::update_content_judgement(&who, id.clone(), LiaisonJudgement::Accepted)?;
            Self::deposit_event(RawEvent::ContentAccepted(id, who));
        }

        fn reject_content(origin, id: T::ContentId) {
            let who = ensure_signed(origin)?;
            Self::update_content_judgement(&who, id.clone(), LiaisonJudgement::Rejected)?;
            Self::deposit_event(RawEvent::ContentRejected(id, who));
        }
    }
}

impl<T: Trait> Module<T> {
    fn update_content_judgement(
        who: &T::AccountId,
        id: T::ContentId,
        judgement: LiaisonJudgement,
    ) -> dispatch::Result {
        // Find the data
        let found = Self::contents(&id).ok_or(MSG_CID_NOT_FOUND);

        // Make sure the liaison matches
        let mut data = found.unwrap();
        ensure!(data.liaison == *who, MSG_LIAISON_REQUIRED);

        // At this point we can update the data.
        data.liaison_judgement = judgement;

        // Update and send event.
        <Contents<T>>::insert(id, data);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::*;

    use system::{self, EventRecord, Phase};

    #[test]
    fn succeed_adding_content() {
        with_default_mock_builder(|| {
            // Register a content name "foo" with 1234 bytes of type 1, which should be recognized.
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_ok());

            // Register the same under a different name
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "bar".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_ok());
        });
    }

    #[test]
    fn fail_adding_content_twice() {
        with_default_mock_builder(|| {
            // Register a content name "foo" with 1234 bytes of type 1, which should be recognized.
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_ok());

            // The second time must fail
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_err());

            // Also from a different origin must fail
            let res = TestDataDirectory::add_content(
                Origin::signed(2),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_err());

            // Also with a different size must fail
            let res = TestDataDirectory::add_content(
                Origin::signed(2),
                1,
                "foo".as_bytes().to_vec(),
                4321,
            );
            assert!(res.is_err());
        });
    }

    #[test]
    fn accept_content_as_liaison() {
        with_default_mock_builder(|| {
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_ok());

            // An appropriate event should have been fired.
            let liaison = *match &System::events().last().unwrap().event {
                MetaEvent::data_directory(data_directory::RawEvent::ContentAdded(
                    _content_id,
                    liaison,
                )) => liaison,
                _ => &0xdeadbeefu64, // invalid value, unlikely to match
            };
            assert_ne!(liaison, 0xdeadbeefu64);

            // Accepting content should not work with some random origin
            let res =
                TestDataDirectory::accept_content(Origin::signed(42), "foo".as_bytes().to_vec());
            assert!(res.is_err());

            // However, with the liaison as origin it should.
            let res = TestDataDirectory::accept_content(
                Origin::signed(liaison),
                "foo".as_bytes().to_vec(),
            );
            assert!(res.is_ok());
        });
    }

    #[test]
    fn reject_content_as_liaison() {
        with_default_mock_builder(|| {
            let res = TestDataDirectory::add_content(
                Origin::signed(1),
                1,
                "foo".as_bytes().to_vec(),
                1234,
            );
            assert!(res.is_ok());

            // An appropriate event should have been fired.
            let liaison = *match &System::events().last().unwrap().event {
                MetaEvent::data_directory(data_directory::RawEvent::ContentAdded(
                    _content_id,
                    liaison,
                )) => liaison,
                _ => &0xdeadbeefu64, // invalid value, unlikely to match
            };
            assert_ne!(liaison, 0xdeadbeefu64);

            // Rejecting content should not work with some random origin
            let res =
                TestDataDirectory::reject_content(Origin::signed(42), "foo".as_bytes().to_vec());
            assert!(res.is_err());

            // However, with the liaison as origin it should.
            let res = TestDataDirectory::reject_content(
                Origin::signed(liaison),
                "foo".as_bytes().to_vec(),
            );
            assert!(res.is_ok());
        });
    }
}
