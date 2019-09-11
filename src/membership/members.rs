use crate::currency::{BalanceOf, GovernanceCurrency};
use codec::{Codec, Decode, Encode};
use rstd::collections::btree_map::BTreeMap;
use rstd::prelude::*;
#[cfg(feature = "std")]
use runtime_io::with_storage;
use runtime_primitives::traits::{MaybeSerializeDebug, Member, SimpleArithmetic};
use srml_support::traits::Currency;
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch, ensure, Parameter, StorageMap, StorageValue,
};
use system::{self, ensure_root, ensure_signed};
use timestamp;

pub trait Trait: system::Trait + GovernanceCurrency + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type MemberId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + PartialEq;

    type PaidTermId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + PartialEq;

    type SubscriptionId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + PartialEq;

    type RoleId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + PartialEq;

    type ActorId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerializeDebug
        + PartialEq;
}

const DEFAULT_FIRST_MEMBER_ID: u32 = 1;
const FIRST_PAID_TERMS_ID: u32 = 1;

// Default paid membership terms
const DEFAULT_PAID_TERM_ID: u32 = 0;
const DEFAULT_PAID_TERM_FEE: u32 = 100; // Can be overidden in genesis config
const DEFAULT_PAID_TERM_TEXT: &str = "Default Paid Term TOS...";

// Default user info constraints
const DEFAULT_MIN_HANDLE_LENGTH: u32 = 5;
const DEFAULT_MAX_HANDLE_LENGTH: u32 = 40;
const DEFAULT_MAX_AVATAR_URI_LENGTH: u32 = 1024;
const DEFAULT_MAX_ABOUT_TEXT_LENGTH: u32 = 2048;

#[derive(Encode, Decode, Eq, PartialEq)]
pub struct ActorInRole<T: Trait> {
    role_id: T::RoleId,
    actor_id: T::ActorId,
}

//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode)]
/// Stored information about a registered user
pub struct Profile<T: Trait> {
    pub handle: Vec<u8>,
    pub avatar_uri: Vec<u8>,
    pub about: Vec<u8>,
    pub registered_at_block: T::BlockNumber,
    pub registered_at_time: T::Moment,
    pub entry: EntryMethod<T>,
    pub suspended: bool,
    pub subscription: Option<T::SubscriptionId>,
    pub controller_account: T::AccountId,
    pub roles: BTreeMap<T::RoleId, Vec<T::ActorId>>,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq)]
/// Structure used to batch user configurable profile information when registering or updating info
pub struct UserInfo {
    pub handle: Option<Vec<u8>>,
    pub avatar_uri: Option<Vec<u8>>,
    pub about: Option<Vec<u8>>,
}

struct CheckedUserInfo {
    handle: Vec<u8>,
    avatar_uri: Vec<u8>,
    about: Vec<u8>,
}

//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Debug, PartialEq)]
pub enum EntryMethod<T: Trait> {
    Paid(T::PaidTermId),
    Screening(T::AccountId),
    Genesis,
}

//#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Eq, PartialEq)]
pub struct PaidMembershipTerms<T: Trait> {
    /// Quantity of native tokens which must be provably burned
    pub fee: BalanceOf<T>,
    /// String of capped length describing human readable conditions which are being agreed upon
    pub text: Vec<u8>,
}

impl<T: Trait> Default for PaidMembershipTerms<T> {
    fn default() -> Self {
        PaidMembershipTerms {
            fee: BalanceOf::<T>::from(DEFAULT_PAID_TERM_FEE),
            text: DEFAULT_PAID_TERM_TEXT.as_bytes().to_vec(),
        }
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Membership {
        /// MemberId's start at this value
        pub FirstMemberId get(first_member_id) config(first_member_id): T::MemberId = T::MemberId::from(DEFAULT_FIRST_MEMBER_ID);

        /// MemberId to assign to next member that is added to the registry
        pub NextMemberId get(next_member_id) build(|config: &GenesisConfig<T>| config.first_member_id): T::MemberId = T::MemberId::from(DEFAULT_FIRST_MEMBER_ID);

        /// Mapping of member ids to their corresponding primary account_id
        pub AccountIdByMemberId get(account_id_by_member_id) : map T::MemberId => T::AccountId;

        /// Mapping of members' primary account ids to their member id.
        pub MemberIdByAccountId get(member_id_by_account_id) : map T::AccountId => Option<T::MemberId>;

        /// Mapping of members' controller account ids to their member id.
        pub MemberIdByControllerAccountId get(member_id_by_controller_account_id) : map T::AccountId => Option<T::MemberId>;

        /// Mapping of member's id to their membership profile
        pub MemberProfile get(member_profile) : map T::MemberId => Option<Profile<T>>;

        /// Registered unique handles and their mapping to their owner
        pub Handles get(handles) : map Vec<u8> => Option<T::MemberId>;

        /// Next paid membership terms id
        pub NextPaidMembershipTermsId get(next_paid_membership_terms_id) : T::PaidTermId = T::PaidTermId::from(FIRST_PAID_TERMS_ID);

        /// Paid membership terms record
        // Remember to add _genesis_phantom_data: std::marker::PhantomData{} to membership
        // genesis config if not providing config() or extra_genesis
        pub PaidMembershipTermsById get(paid_membership_terms_by_id) build(|config: &GenesisConfig<T>| {
            // This method only gets called when initializing storage, and is
            // compiled as native code. (Will be called when building `raw` chainspec)
            // So it can't be relied upon to initialize storage for runtimes updates.
            // Initialization for updated runtime is done in run_migration()
            let mut terms: PaidMembershipTerms<T> = Default::default();
            terms.fee = config.default_paid_membership_fee;
            vec![(T::PaidTermId::from(DEFAULT_PAID_TERM_ID), terms)]
        }) : map T::PaidTermId => Option<PaidMembershipTerms<T>>;

        /// Active Paid membership terms
        pub ActivePaidMembershipTerms get(active_paid_membership_terms) : Vec<T::PaidTermId> = vec![T::PaidTermId::from(DEFAULT_PAID_TERM_ID)];

        /// Is the platform is accepting new members or not
        pub NewMembershipsAllowed get(new_memberships_allowed) : bool = true;

        pub ScreeningAuthority get(screening_authority) : Option<T::AccountId>;

        // User Input Validation parameters - do these really need to be state variables
        // I don't see a need to adjust these in future?
        pub MinHandleLength get(min_handle_length) : u32 = DEFAULT_MIN_HANDLE_LENGTH;
        pub MaxHandleLength get(max_handle_length) : u32 = DEFAULT_MAX_HANDLE_LENGTH;
        pub MaxAvatarUriLength get(max_avatar_uri_length) : u32 = DEFAULT_MAX_AVATAR_URI_LENGTH;
        pub MaxAboutTextLength get(max_about_text_length) : u32 = DEFAULT_MAX_ABOUT_TEXT_LENGTH;

        pub MembershipIdByActorInRole get(membership_id_by_actor_in_role): map ActorInRole<T> => T::MemberId;
    }
    add_extra_genesis {
        config(default_paid_membership_fee): BalanceOf<T>;
        config(members) : Vec<(T::AccountId, Vec<u8>, Vec<u8>, Vec<u8>)>;
        build(|
            storage: &mut (runtime_primitives::StorageOverlay, runtime_primitives::ChildrenStorageOverlay),
            config: &GenesisConfig<T>
        | {
            with_storage(storage, || {
                for (who, handle, avatar_uri, about) in &config.members {
                    let user_info = CheckedUserInfo {
                        handle: handle.clone(), avatar_uri: avatar_uri.clone(), about: about.clone()
                    };
                    <Module<T>>::insert_member(&who, &user_info, EntryMethod::Genesis);
                }
            });
        });

    }
}

decl_event! {
    pub enum Event<T> where
      <T as system::Trait>::AccountId,
      <T as Trait>::MemberId,
      <T as Trait>::RoleId,
      <T as Trait>::ActorId, {
        MemberRegistered(MemberId, AccountId),
        MemberUpdatedAboutText(MemberId),
        MemberUpdatedAvatar(MemberId),
        MemberUpdatedHandle(MemberId),
        MemberSetPrimaryKey(MemberId, AccountId),
        MemberSetControllerKey(MemberId, AccountId),
        MemberRegisteredRole(MemberId, RoleId, ActorId),
        MemberUnregisteredRole(MemberId, RoleId, ActorId),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;

        /// Non-members can buy membership
        pub fn buy_membership(origin, paid_terms_id: T::PaidTermId, user_info: UserInfo) {
            let who = ensure_signed(origin)?;

            // make sure we are accepting new memberships
            ensure!(Self::new_memberships_allowed(), "new members not allowed");

            // ensure key not associated with an existing membership
            Self::ensure_not_member(&who)?;

            // TODO: ensure account doesn't have any locks, is not a signing account of another member
            // what other restrictions should we apply if any at all?

            // ensure paid_terms_id is active
            let terms = Self::ensure_active_terms_id(paid_terms_id)?;

            // ensure enough free balance to cover terms fees
            ensure!(T::Currency::can_slash(&who, terms.fee), "not enough balance to buy membership");

            let user_info = Self::check_user_registration_info(user_info)?;

            // ensure handle is not already registered
            Self::ensure_unique_handle(&user_info.handle)?;

            let _ = T::Currency::slash(&who, terms.fee);
            let member_id = Self::insert_member(&who, &user_info, EntryMethod::Paid(paid_terms_id));

            Self::deposit_event(RawEvent::MemberRegistered(member_id, who.clone()));
        }

        /// Change member's about text
        pub fn change_member_about_text(origin, text: Vec<u8>) {
            let controller = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_controller_account(&controller)?;
            Self::_change_member_about_text(member_id, &text)?;
        }

        /// Change member's avatar
        pub fn change_member_avatar(origin, uri: Vec<u8>) {
            let controller = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_controller_account(&controller)?;
            Self::_change_member_avatar(member_id, &uri)?;
        }

        /// Change member's handle. Will ensure new handle is unique and old one will be available
        /// for other members to use.
        pub fn change_member_handle(origin, handle: Vec<u8>) {
            let controller = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_controller_account(&controller)?;
            Self::_change_member_handle(member_id, handle)?;
        }

        /// Update member's all or some of handle, avatar and about text.
        pub fn update_profile(origin, user_info: UserInfo) {
            let controller = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_controller_account(&controller)?;

            if let Some(uri) = user_info.avatar_uri {
                Self::_change_member_avatar(member_id, &uri)?;
            }
            if let Some(about) = user_info.about {
                Self::_change_member_about_text(member_id, &about)?;
            }
            if let Some(handle) = user_info.handle {
                Self::_change_member_handle(member_id, handle)?;
            }
        }

        pub fn set_controller_key(origin, controller: T::AccountId) {
            let master = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_primary_account(&master)?;

            let mut profile = Self::ensure_profile(member_id)?;

            // ensure new key is not used by someone else (master or controller)
            ensure!(!<MemberIdByControllerAccountId<T>>::exists(&controller), "account already paired with member");
            ensure!(!<MemberIdByAccountId<T>>::exists(&controller), "account already paired with member");

            <MemberIdByControllerAccountId<T>>::remove(&profile.controller_account);
            <MemberIdByControllerAccountId<T>>::insert(&profile.controller_account, member_id);
            profile.controller_account = controller.clone();
            <MemberProfile<T>>::insert(member_id, profile);
            Self::deposit_event(RawEvent::MemberSetControllerKey(member_id, controller));
        }

        pub fn set_primary_key(origin, new_primary: T::AccountId) {
            let old_primary = ensure_signed(origin)?;
            let member_id = Self::ensure_is_member_primary_account(&old_primary)?;

            // ensure new key not is used by someone else (master or controller)
            ensure!(!<MemberIdByControllerAccountId<T>>::exists(&new_primary), "account already paired with member");
            ensure!(!<MemberIdByAccountId<T>>::exists(&new_primary), "account already paired with member");

            // update mappings
            <AccountIdByMemberId<T>>::insert(member_id, new_primary.clone());
            <MemberIdByAccountId<T>>::remove(&old_primary);
            <MemberIdByAccountId<T>>::insert(&new_primary, member_id);
            Self::deposit_event(RawEvent::MemberSetPrimaryKey(member_id, new_primary));
        }

        pub fn add_screened_member(origin, new_member: T::AccountId, user_info: UserInfo) {
            // ensure sender is screening authority
            let sender = ensure_signed(origin)?;

            if let Some(screening_authority) = Self::screening_authority() {
                ensure!(sender == screening_authority, "not screener");
            } else {
                // no screening authority defined. Cannot accept this request
                return Err("no screening authority defined");
            }

            // make sure we are accepting new memberships
            ensure!(Self::new_memberships_allowed(), "new members not allowed");

            // ensure key not associated with an existing membership
            Self::ensure_not_member(&new_member)?;

            // TODO: ensure account doesn't have any locks, is not a signing account of another member
            // what other restrictions should we apply if any at all?

            let user_info = Self::check_user_registration_info(user_info)?;

            // ensure handle is not already registered
            Self::ensure_unique_handle(&user_info.handle)?;

            let member_id = Self::insert_member(&new_member, &user_info, EntryMethod::Screening(sender));

            Self::deposit_event(RawEvent::MemberRegistered(member_id, new_member.clone()));
        }

        pub fn set_screening_authority(origin, authority: T::AccountId) {
            ensure_root(origin)?;
            <ScreeningAuthority<T>>::put(authority);
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn is_active_member(who: &T::AccountId) -> bool {
        match Self::ensure_is_member_primary_account(who)
            .and_then(|member_id| Self::ensure_profile(member_id))
        {
            Ok(profile) => !profile.suspended,
            Err(_err) => false,
        }
    }

    pub fn primary_account_by_member_id(id: T::MemberId) -> Result<T::AccountId, &'static str> {
        if <AccountIdByMemberId<T>>::exists(&id) {
            Ok(Self::account_id_by_member_id(&id))
        } else {
            Err("member id doesn't exist")
        }
    }

    fn ensure_not_member(who: &T::AccountId) -> dispatch::Result {
        ensure!(
            !<MemberIdByAccountId<T>>::exists(who),
            "account already associated with a membership"
        );
        Ok(())
    }

    pub fn ensure_is_member_primary_account(
        who: &T::AccountId,
    ) -> Result<T::MemberId, &'static str> {
        let member_id =
            Self::member_id_by_account_id(who).ok_or("no member id found for accountid")?;
        Ok(member_id)
    }

    pub fn ensure_is_member_controller_account(
        who: &T::AccountId,
    ) -> Result<T::MemberId, &'static str> {
        let member_id = Self::member_id_by_controller_account_id(who)
            .ok_or("no member id found for accountid")?;
        Ok(member_id)
    }

    fn ensure_profile(id: T::MemberId) -> Result<Profile<T>, &'static str> {
        let profile = Self::member_profile(&id).ok_or("member profile not found")?;
        Ok(profile)
    }

    pub fn get_profile_by_primary_account(id: &T::AccountId) -> Option<Profile<T>> {
        if let Ok(member_id) = Self::ensure_is_member_primary_account(id) {
            // This option _must_ be set
            Self::member_profile(&member_id)
        } else {
            None
        }
    }

    fn ensure_active_terms_id(
        terms_id: T::PaidTermId,
    ) -> Result<PaidMembershipTerms<T>, &'static str> {
        let active_terms = Self::active_paid_membership_terms();
        ensure!(
            active_terms.iter().any(|&id| id == terms_id),
            "paid terms id not active"
        );
        let terms = Self::paid_membership_terms_by_id(terms_id)
            .ok_or("paid membership term id does not exist")?;
        Ok(terms)
    }

    fn ensure_unique_handle(handle: &Vec<u8>) -> dispatch::Result {
        ensure!(!<Handles<T>>::exists(handle), "handle already registered");
        Ok(())
    }

    fn validate_handle(handle: &Vec<u8>) -> dispatch::Result {
        ensure!(
            handle.len() >= Self::min_handle_length() as usize,
            "handle too short"
        );
        ensure!(
            handle.len() <= Self::max_handle_length() as usize,
            "handle too long"
        );
        Ok(())
    }

    fn validate_text(text: &Vec<u8>) -> Vec<u8> {
        let mut text = text.clone();
        text.truncate(Self::max_about_text_length() as usize);
        text
    }

    fn validate_avatar(uri: &Vec<u8>) -> dispatch::Result {
        ensure!(
            uri.len() <= Self::max_avatar_uri_length() as usize,
            "avatar uri too long"
        );
        Ok(())
    }

    /// Basic user input validation
    fn check_user_registration_info(user_info: UserInfo) -> Result<CheckedUserInfo, &'static str> {
        // Handle is required during registration
        let handle = user_info
            .handle
            .ok_or("handle must be provided during registration")?;
        Self::validate_handle(&handle)?;

        let about = Self::validate_text(&user_info.about.unwrap_or_default());
        let avatar_uri = user_info.avatar_uri.unwrap_or_default();
        Self::validate_avatar(&avatar_uri)?;

        Ok(CheckedUserInfo {
            handle,
            avatar_uri,
            about,
        })
    }

    fn insert_member(
        who: &T::AccountId,
        user_info: &CheckedUserInfo,
        entry_method: EntryMethod<T>,
    ) -> T::MemberId {
        let new_member_id = Self::next_member_id();

        let profile: Profile<T> = Profile {
            handle: user_info.handle.clone(),
            avatar_uri: user_info.avatar_uri.clone(),
            about: user_info.about.clone(),
            registered_at_block: <system::Module<T>>::block_number(),
            registered_at_time: <timestamp::Module<T>>::now(),
            entry: entry_method,
            suspended: false,
            subscription: None,
            roles: BTreeMap::new(),
            controller_account: who.clone(),
        };

        <MemberIdByAccountId<T>>::insert(who.clone(), new_member_id);
        <MemberIdByControllerAccountId<T>>::insert(who.clone(), new_member_id);
        <AccountIdByMemberId<T>>::insert(new_member_id, who.clone());
        <MemberProfile<T>>::insert(new_member_id, profile);
        <Handles<T>>::insert(user_info.handle.clone(), new_member_id);
        <NextMemberId<T>>::mutate(|n| {
            *n += T::MemberId::from(1);
        });

        new_member_id
    }

    fn _change_member_about_text(id: T::MemberId, text: &Vec<u8>) -> dispatch::Result {
        let mut profile = Self::ensure_profile(id)?;
        let text = Self::validate_text(text);
        profile.about = text;
        Self::deposit_event(RawEvent::MemberUpdatedAboutText(id));
        <MemberProfile<T>>::insert(id, profile);
        Ok(())
    }

    fn _change_member_avatar(id: T::MemberId, uri: &Vec<u8>) -> dispatch::Result {
        let mut profile = Self::ensure_profile(id)?;
        Self::validate_avatar(uri)?;
        profile.avatar_uri = uri.clone();
        Self::deposit_event(RawEvent::MemberUpdatedAvatar(id));
        <MemberProfile<T>>::insert(id, profile);
        Ok(())
    }

    fn _change_member_handle(id: T::MemberId, handle: Vec<u8>) -> dispatch::Result {
        let mut profile = Self::ensure_profile(id)?;
        Self::validate_handle(&handle)?;
        Self::ensure_unique_handle(&handle)?;
        <Handles<T>>::remove(&profile.handle);
        <Handles<T>>::insert(handle.clone(), id);
        profile.handle = handle;
        Self::deposit_event(RawEvent::MemberUpdatedHandle(id));
        <MemberProfile<T>>::insert(id, profile);
        Ok(())
    }

    // Member role registraion
    pub fn member_is_in_role(member_id: T::MemberId, role_id: T::RoleId) -> bool {
        Self::ensure_profile(member_id)
            .ok()
            .and_then(|profile| {
                if let Some(actor_ids) = profile.roles.get(&role_id) {
                    Some(actor_ids.len() > 0)
                } else {
                    None
                }
            })
            .unwrap_or(false)
    }

    pub fn can_register_role_on_member(
        member_id: T::MemberId,
        role_id: T::RoleId,
        actor_id: T::ActorId,
    ) -> Result<(), &'static str> {
        // For now default policy across all roles is:
        // members can only have a single actor instance per role
        // members can enter any roles
        // no limit on total number of roles a member can enter
        // Note: Role specific policies, for example "member can only enter council role once at a time"
        // should be enforced by the council module (client modules)
        ensure!(
            !Self::member_is_in_role(member_id, role_id),
            "member already in role"
        );

        // ensure is active member
        let profile = Self::ensure_profile(member_id)?;
        ensure!(!profile.suspended, "suspended members can't enter a role");

        // guard against duplicate ActorInRole
        let actor_in_role = ActorInRole { role_id, actor_id };
        ensure!(
            !<MembershipIdByActorInRole<T>>::exists(&actor_in_role),
            "role actor already exists"
        );
        Ok(())
    }

    pub fn register_role_on_member(
        member_id: T::MemberId,
        role_id: T::RoleId,
        actor_id: T::ActorId,
    ) -> Result<(), &'static str> {
        ensure!(
            Self::can_register_role_on_member(member_id, role_id, actor_id).is_ok(),
            "registering role not allowed"
        );

        let mut profile = Self::ensure_profile(member_id)?;
        let mut new_ids = vec![actor_id];

        if let Some(current_ids) = profile.roles.get_mut(&role_id) {
            current_ids.append(&mut new_ids);
        } else {
            profile.roles.insert(role_id, new_ids);
        }
        <MemberProfile<T>>::insert(member_id, profile);
        <MembershipIdByActorInRole<T>>::insert(ActorInRole { role_id, actor_id }, member_id);
        Self::deposit_event(RawEvent::MemberRegisteredRole(member_id, role_id, actor_id));
        Ok(())
    }

    pub fn can_unregister_role_on_member(
        member_id: T::MemberId,
        role_id: T::RoleId,
        actor_id: T::ActorId,
    ) -> Result<(), &'static str> {
        let actor_in_role = ActorInRole { role_id, actor_id };
        ensure!(
            <MembershipIdByActorInRole<T>>::exists(&actor_in_role),
            "role actor not found"
        );
        ensure!(
            <MembershipIdByActorInRole<T>>::get(&actor_in_role) == member_id,
            "role actor not for member"
        );
        Ok(())
    }

    pub fn unregister_role_on_member(
        member_id: T::MemberId,
        role_id: T::RoleId,
        actor_id: T::ActorId,
    ) -> Result<(), &'static str> {
        ensure!(
            Self::can_unregister_role_on_member(member_id, role_id, actor_id).is_ok(),
            "unregistering role not allowed"
        );

        let mut profile = Self::ensure_profile(member_id)?;

        if let Some(current_ids) = profile.roles.get_mut(&role_id) {
            //current_ids.remove_item(&actor_id); // unstable nightly feature!
            current_ids.retain(|id| *id != actor_id);
            <MemberProfile<T>>::insert(member_id, profile);
        }

        <MembershipIdByActorInRole<T>>::remove(ActorInRole { role_id, actor_id });
        Self::deposit_event(RawEvent::MemberUnregisteredRole(
            member_id, role_id, actor_id,
        ));
        Ok(())
    }
}
