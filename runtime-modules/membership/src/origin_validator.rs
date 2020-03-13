use rstd::marker::PhantomData;

use srml_support::print;
use system::ensure_signed;

/// Abstract validator for the origin(account_id) and actor_id (eg.: thread author id).
pub trait ActorOriginValidator<Origin, ActorId, AccountId> {
    /// Check for valid combination of origin and actor_id
    fn ensure_actor_origin(
        origin: Origin,
        actor_id: ActorId,
        error: &'static str,
    ) -> Result<AccountId, &'static str>;
}

/// Member of the Joystream organization
pub type MemberId<T> = <T as crate::members::Trait>::MemberId;

/// Default discussion system actor origin validator. Valid for both thread and post authors.
pub struct MembershipOriginValidator<T> {
    marker: PhantomData<T>,
}

impl<T> MembershipOriginValidator<T> {
    /// Create ThreadPostActorOriginValidator instance
    pub fn new() -> Self {
        MembershipOriginValidator {
            marker: PhantomData,
        }
    }
}

impl<T: crate::members::Trait>
    ActorOriginValidator<<T as system::Trait>::Origin, MemberId<T>, <T as system::Trait>::AccountId>
    for MembershipOriginValidator<T>
{
    /// Check for valid combination of origin and actor_id. Actor_id should be valid member_id of
    /// the membership module
    fn ensure_actor_origin(
        origin: <T as system::Trait>::Origin,
        actor_id: MemberId<T>,
        error: &'static str,
    ) -> Result<<T as system::Trait>::AccountId, &'static str> {
        // check valid signed account_id
        let account_id = ensure_signed(origin)?;

        // check whether actor_id belongs to the registered member
        let profile_result = <crate::members::Module<T>>::ensure_profile(actor_id);

        if let Ok(profile) = profile_result {
            print("profile");
            // whether the account_id belongs to the actor
            if profile.root_account == account_id || profile.controller_account == account_id {
                return Ok(account_id);
            }
        }

        Err(error)
    }
}

#[cfg(test)]
mod tests {

    use crate::members::UserInfo;
    use crate::mock::{Test, TestExternalitiesBuilder};
    use crate::origin_validator::{ActorOriginValidator, MembershipOriginValidator};
    use system::RawOrigin;

    type Membership = crate::members::Module<Test>;

    pub fn initial_test_ext() -> runtime_io::TestExternalities {
        const DEFAULT_FEE: u64 = 500;
        let initial_members = [1, 2, 3];

        TestExternalitiesBuilder::<Test>::default()
            .set_membership_config(
                crate::genesis::GenesisConfigBuilder::default()
                    .default_paid_membership_fee(DEFAULT_FEE)
                    .members(initial_members.to_vec())
                    .build(),
            )
            .build()
    }

    #[test]
    fn membership_origin_validator_fails_with_unregistered_member() {
        initial_test_ext().execute_with(|| {
            let origin = RawOrigin::Signed(1);
            let member_id = 1;
            let error = "Error";

            let validation_result = MembershipOriginValidator::<Test>::ensure_actor_origin(
                origin.into(),
                member_id,
                error,
            );

            assert_eq!(validation_result, Err(error));
        });
    }

    #[test]
    fn membership_origin_validator_succeeds() {
        initial_test_ext().execute_with(|| {
            let account_id = 1;
            let origin = RawOrigin::Signed(account_id);
            let member_id = 0;
            let error = "Error";
            let authority_account_id = 10;
            Membership::set_screening_authority(RawOrigin::Root.into(), authority_account_id)
                .unwrap();

            Membership::add_screened_member(
                RawOrigin::Signed(authority_account_id).into(),
                account_id,
                UserInfo {
                    handle: Some(b"handle".to_vec()),
                    avatar_uri: None,
                    about: None,
                },
            )
            .unwrap();

            let validation_result = MembershipOriginValidator::<Test>::ensure_actor_origin(
                origin.into(),
                member_id,
                error,
            );

            assert_eq!(validation_result, Ok(account_id));
        });
    }

    #[test]
    fn membership_origin_validator_fails_with_incompatible_account_id_and_member_id() {
        initial_test_ext().execute_with(|| {
            let account_id = 1;
            let member_id = 0;
            let error = "Error";
            let authority_account_id = 10;
            Membership::set_screening_authority(RawOrigin::Root.into(), authority_account_id)
                .unwrap();

            Membership::add_screened_member(
                RawOrigin::Signed(authority_account_id).into(),
                account_id,
                UserInfo {
                    handle: Some(b"handle".to_vec()),
                    avatar_uri: None,
                    about: None,
                },
            )
            .unwrap();

            let invalid_account_id = 2;
            let validation_result = MembershipOriginValidator::<Test>::ensure_actor_origin(
                RawOrigin::Signed(invalid_account_id).into(),
                member_id,
                error,
            );

            assert_eq!(validation_result, Err(error));
        });
    }
}
