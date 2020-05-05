// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod constraints;
mod types;
#[cfg(test)]
mod tests;

use rstd::collections::btree_set::BTreeSet;
use rstd::vec::Vec;
use rstd::prelude::*;
use sr_primitives::traits::{One, Zero, EnsureOrigin};
use srml_support::traits::{Currency, ExistenceRequirement, WithdrawReasons};
use srml_support::{decl_module, decl_storage, decl_event, dispatch, ensure};
use system::{ensure_root, ensure_signed, RawOrigin};

use constraints::InputValidationLengthConstraint;
use types::{CuratorOpening, Lead, OpeningPolicyCommitment, CuratorApplication};

/*
+ add_curator_opening
- accept_curator_applications
- begin_curator_applicant_review
- fill_curator_opening
- withdraw_curator_application
- terminate_curator_application
- apply_on_curator_opening
*/


//TODO: convert messages to the decl_error! entries
pub static MSG_ORIGIN_IS_NOT_LEAD: &str = "Origin is not lead";
pub static MSG_CURRENT_LEAD_NOT_SET: &str = "Current lead is not set";
pub static MSG_CURRENT_LEAD_ALREADY_SET: &str = "Current lead is already set";
pub static MSG_IS_NOT_LEAD_ACCOUNT: &str = "Not a lead account";
pub static MSG_CHANNEL_DESCRIPTION_TOO_SHORT: &str = "Channel description too short";
pub static MSG_CHANNEL_DESCRIPTION_TOO_LONG: &str = "Channel description too long";
pub static MSG_CURATOR_OPENING_DOES_NOT_EXIST: &str = "Curator opening does not exist";
pub static MSG_INSUFFICIENT_BALANCE_TO_APPLY: &str = "Insufficient balance to apply";
pub static MSG_APPLY_ON_CURATOR_OPENING_UNSIGNED_ORIGIN: &str = "Unsigned origin";
pub static MSG_APPLY_ON_CURATOR_OPENING_MEMBER_ID_INVALID: &str = "Member id is invalid";
pub static MSG_APPLY_ON_CURATOR_OPENING_SIGNER_NOT_CONTROLLER_ACCOUNT: &str =
    "Signer does not match controller account";
pub static MSG_ORIGIN_IS_NIETHER_MEMBER_CONTROLLER_OR_ROOT: &str =
    "Origin must be controller or root account of member";
pub static MSG_MEMBER_HAS_ACTIVE_APPLICATION_ON_OPENING: &str =
    "Member already has an active application on the opening";
pub static MSG_CURATOR_APPLICATION_TEXT_TOO_LONG: &str = "Curator application text too long";
pub static MSG_CURATOR_APPLICATION_TEXT_TOO_SHORT: &str = "Curator application text too short";
pub static MSG_INSUFFICIENT_BALANCE_TO_COVER_STAKE: &str = "Insuffieicnt balance to cover stake";

/// Alias for the _Lead_ type
pub type LeadOf<T> =
Lead<<T as membership::members::Trait>::MemberId, <T as system::Trait>::AccountId>;

// Workaround for BTreeSet type
pub type CuratorApplicationIdSet<T> = BTreeSet<CuratorApplicationId<T>>;

/// Type for the identifier for an opening for a curator.
pub type CuratorOpeningId<T> = <T as hiring::Trait>::OpeningId;

/// Type for the identifier for an application as a curator.
pub type CuratorApplicationId<T> = <T as hiring::Trait>::ApplicationId;

/// Balance type of runtime
pub type BalanceOf<T> =
    <<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// Balance type of runtime
pub type CurrencyOf<T> = <T as stake::Trait>::Currency;

/// Negative imbalance of runtime.
pub type NegativeImbalance<T> =
<<T as stake::Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

/// The bureaucracy main _Trait_
pub trait Trait<I: Instance>:
    system::Trait + membership::members::Trait + hiring::Trait
{
    /// Engine event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
    /// Proposals engine events
    pub enum Event<T, I>
    where
        <T as membership::members::Trait>::MemberId,
        <T as system::Trait>::AccountId
    {
        /// Emits on setting the leader.
        /// Params:
        /// - Member id of the leader.
        /// - Role account id of the leader.
        LeaderSet(MemberId, AccountId),
    }
);

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance> as Bureaucracy {
        /// The current lead.
        pub CurrentLead get(current_lead) : Option<LeadOf<T>>;

        /// Next identifier value for new curator opening.
        pub NextCuratorOpeningId get(next_curator_opening_id): CuratorOpeningId<T>;

        /// Maps identifier to curator opening.
        pub CuratorOpeningById get(curator_opening_by_id): linked_map CuratorOpeningId<T> => CuratorOpening<T::OpeningId, T::BlockNumber, BalanceOf<T>, CuratorApplicationId<T>>;

        pub OpeningHumanReadableText get(opening_human_readable_text): InputValidationLengthConstraint;


        /// Maps identifier to curator application on opening.
        pub CuratorApplicationById get(curator_application_by_id) : linked_map CuratorApplicationId<T> => CuratorApplication<T::AccountId, CuratorOpeningId<T>, T::MemberId, T::ApplicationId>;

        /// Next identifier value for new curator application.
        pub NextCuratorApplicationId get(next_curator_application_id) : CuratorApplicationId<T>;

        pub CuratorApplicationHumanReadableText get(curator_application_human_readable_text) : InputValidationLengthConstraint;

    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance> for enum Call where origin: T::Origin {
        /// Default deposit_event() handler
        fn deposit_event() = default;


        /// Introduce a lead when one is not currently set.
        pub fn set_lead(origin, member_id: T::MemberId, role_account_id: T::AccountId) -> dispatch::Result {
            ensure_root(origin)?;

            // Construct lead
            let new_lead = Lead {
                member_id,
                role_account_id: role_account_id.clone(),
            };

            // mutation

            // Update current lead
            <CurrentLead<T, I>>::put(new_lead);

            // Trigger an event
            Self::deposit_event(RawEvent::LeaderSet(member_id, role_account_id));


            Ok(())
        }

            /// Add an opening for a curator role.
        pub fn add_curator_opening(_origin, activate_at: hiring::ActivateOpeningAt<T::BlockNumber>, commitment: OpeningPolicyCommitment<T::BlockNumber, BalanceOf<T>>, human_readable_text: Vec<u8>)  {

            // Ensure lead is set and is origin signer
            //Self::ensure_origin_is_set_lead(origin)?;

            Self::ensure_opening_human_readable_text_is_valid(&human_readable_text)?;

            // Add opening
            // NB: This call can in principle fail, because the staking policies
            // may not respect the minimum currency requirement.

            let policy_commitment = commitment.clone();

            // let opening_id = ensure_on_wrapped_error!(
            //     hiring::Module::<T>::add_opening(
            //         activate_at,
            //         commitment.max_review_period_length,
            //         commitment.application_rationing_policy,
            //         commitment.application_staking_policy,
            //         commitment.role_staking_policy,
            //         human_readable_text,
            //     ))?;

            let opening_id = hiring::Module::<T>::add_opening(
                activate_at,
                commitment.max_review_period_length,
                commitment.application_rationing_policy,
                commitment.application_staking_policy,
                commitment.role_staking_policy,
                human_readable_text,
            ).unwrap(); //TODO

            //
            // == MUTATION SAFE ==
            //

            let new_curator_opening_id = NextCuratorOpeningId::<T, I>::get();

            // Create and add curator opening.
            let new_opening_by_id = CuratorOpening::<CuratorOpeningId<T>, T::BlockNumber, BalanceOf<T>, CuratorApplicationId<T>> {
                opening_id : opening_id,
                curator_applications: BTreeSet::new(),
                policy_commitment: policy_commitment
            };

            CuratorOpeningById::<T, I>::insert(new_curator_opening_id, new_opening_by_id);

            // Update NextCuratorOpeningId
            NextCuratorOpeningId::<T, I>::mutate(|id| *id += <CuratorOpeningId<T> as One>::one());

            // Trigger event
            //Self::deposit_event(RawEvent::CuratorOpeningAdded(new_curator_opening_id));
    }

            /// Begin accepting curator applications to an opening that is active.
        pub fn accept_curator_applications(_origin, curator_opening_id: CuratorOpeningId<T>)  {

            // Ensure lead is set and is origin signer
            //Self::ensure_origin_is_set_lead(origin)?;

            // Ensure opening exists in this working group
            // NB: Even though call to hiring module will have implicit check for
            // existence of opening as well, this check is to make sure that the opening is for
            // this working group, not something else.
            let (curator_opening, _opening) = Self::ensure_curator_opening_exists(&curator_opening_id)?;

            // Attempt to begin accepting applications
            // NB: Combined ensure check and mutation in hiring module

            // ensure_on_wrapped_error!(
            //     hiring::Module::<T>::begin_accepting_applications(curator_opening.opening_id)
            //     )?;


            // mutation

            hiring::Module::<T>::begin_accepting_applications(curator_opening.opening_id).unwrap(); //TODO


            // Trigger event
            // Self::deposit_event(RawEvent::AcceptedCuratorApplications(curator_opening_id));
        }




           /// Apply on a curator opening.
        pub fn apply_on_curator_opening(
            origin,
            member_id: T::MemberId,
            curator_opening_id: CuratorOpeningId<T>,
            role_account: T::AccountId,
            opt_role_stake_balance: Option<BalanceOf<T>>,
            opt_application_stake_balance: Option<BalanceOf<T>>,
            human_readable_text: Vec<u8>
        ) {
            // Ensure origin which will server as the source account for staked funds is signed
            let source_account = ensure_signed(origin)?;

            // In absense of a more general key delegation system which allows an account with some funds to
            // grant another account permission to stake from its funds, the origin of this call must have the funds
            // and cannot specify another arbitrary account as the source account.
            // Ensure the source_account is either the controller or root account of member with given id
            ensure!(
                membership::members::Module::<T>::ensure_member_controller_account(&source_account, &member_id).is_ok() ||
                membership::members::Module::<T>::ensure_member_root_account(&source_account, &member_id).is_ok(),
                MSG_ORIGIN_IS_NIETHER_MEMBER_CONTROLLER_OR_ROOT
            );

            // Ensure curator opening exists
            let (curator_opening, _opening) = Self::ensure_curator_opening_exists(&curator_opening_id)?;

            // Ensure new owner can actually become a curator
            //let (_member_as_curator, _new_curator_id) = Self::ensure_can_register_curator_role_on_member(&member_id)?;

            // Ensure that there is sufficient balance to cover stake proposed
            Self::ensure_can_make_stake_imbalance(
                vec![&opt_role_stake_balance, &opt_application_stake_balance],
                &source_account)
                .map_err(|_err| MSG_INSUFFICIENT_BALANCE_TO_APPLY)?;

            // Ensure application text is valid
            Self::ensure_curator_application_text_is_valid(&human_readable_text)?;

            // Ensure application can actually be added
            // ensure_on_wrapped_error!(
            //     hiring::Module::<T>::ensure_can_add_application(curator_opening.opening_id, opt_role_stake_balance, opt_application_stake_balance)
            // )?;
                hiring::Module::<T>::ensure_can_add_application(curator_opening.opening_id, opt_role_stake_balance, opt_application_stake_balance)
            .unwrap(); //TODO

            // Ensure member does not have an active application to this opening
            Self::ensure_member_has_no_active_application_on_opening(
                curator_opening.curator_applications,
                member_id
            )?;

            //
            // == MUTATION SAFE ==
            //

            // Make imbalances for staking
            let opt_role_stake_imbalance = Self::make_stake_opt_imbalance(&opt_role_stake_balance, &source_account);
            let opt_application_stake_imbalance = Self::make_stake_opt_imbalance(&opt_application_stake_balance, &source_account);

            // Call hiring module to add application
            let add_application_result = hiring::Module::<T>::add_application(
                curator_opening.opening_id,
                opt_role_stake_imbalance,
                opt_application_stake_imbalance,
                human_readable_text
            );

            // Has to hold
            assert!(add_application_result.is_ok());

            let application_id = add_application_result.unwrap().application_id_added;

            // Get id of new curator application
            let new_curator_application_id = NextCuratorApplicationId::<T, I>::get();

            // Make curator application
            let curator_application = CuratorApplication::new(&role_account, &curator_opening_id, &member_id, &application_id);

            // Store application
            CuratorApplicationById::<T, I>::insert(new_curator_application_id, curator_application);

            // Update next curator application identifier value
            NextCuratorApplicationId::<T, I>::mutate(|id| *id += <CuratorApplicationId<T> as One>::one());

            // Add application to set of application in curator opening
            CuratorOpeningById::<T, I>::mutate(curator_opening_id, |curator_opening| {
                curator_opening.curator_applications.insert(new_curator_application_id);
            });

            // Trigger event
            //Self::deposit_event(RawEvent::AppliedOnCuratorOpening(curator_opening_id, new_curator_application_id));
        }
    }
}

impl<Origin, T, I> EnsureOrigin<Origin> for Module<T, I>
    where
        Origin: Into<Result<RawOrigin<T::AccountId>, Origin>> + From<RawOrigin<T::AccountId>>,
        T: Trait<I>,
        I: Instance,
{
    type Success = ();

    fn try_origin(o: Origin) -> Result<Self::Success, Origin> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(account_id) => {
                Self::ensure_is_lead_account(account_id).map_err(|_| RawOrigin::None.into())
            }
            _ => Err(RawOrigin::None.into()),
        })
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    /// Checks that provided lead account id belongs to the current bureaucracy leader
    pub fn ensure_is_lead_account(lead_account_id: T::AccountId) -> Result<(), &'static str> {
        let lead = <CurrentLead<T, I>>::get();

        if let Some(lead) = lead {
            if lead.role_account_id != lead_account_id {
                return Err(MSG_IS_NOT_LEAD_ACCOUNT);
            }
        } else {
            return Err(MSG_CURRENT_LEAD_NOT_SET);
        }

        Ok(())
    }

    fn ensure_opening_human_readable_text_is_valid(text: &Vec<u8>) -> dispatch::Result {
        <OpeningHumanReadableText<I>>::get().ensure_valid(
            text.len(),
            MSG_CHANNEL_DESCRIPTION_TOO_SHORT,
            MSG_CHANNEL_DESCRIPTION_TOO_LONG,
        )
    }

    // fn ensure_origin_is_set_lead(
    //     origin: T::Origin,
    // ) -> Result<
    //     (
    //         LeadId<T>,
    //         Lead<T::AccountId, T::RewardRelationshipId, T::BlockNumber>,
    //     ),
    //     &'static str,
    // > {
    //     // Ensure lead is actually set
    //     let (lead_id, lead) = Self::ensure_lead_is_set()?;
    //
    //     // Ensure is signed
    //     let signer = ensure_signed(origin)?;
    //
    //     // Ensure signer is lead
    //     ensure!(signer == lead.role_account, MSG_ORIGIN_IS_NOT_LEAD);
    //
    //     Ok((lead_id, lead))
    // }

    // pub fn ensure_lead_is_set() -> Result<
    //     (
    //         LeadId<T>,
    //         Lead<T::AccountId, T::RewardRelationshipId, T::BlockNumber>,
    //     ),
    //     &'static str,
    // > {
    //     // Ensure lead id is set
    //     let lead_id = Self::ensure_lead_id_set()?;
    //
    //     // If so, grab actual lead
    //     let lead = <LeadById<T, I>>::get(lead_id);
    //
    //     // and return both
    //     Ok((lead_id, lead))
    // }




    fn ensure_curator_opening_exists(
        curator_opening_id: &CuratorOpeningId<T>,
    ) -> Result<
        (
            CuratorOpening<T::OpeningId, T::BlockNumber, BalanceOf<T>, CuratorApplicationId<T>>,
            hiring::Opening<BalanceOf<T>, T::BlockNumber, <T as hiring::Trait>::ApplicationId>,
        ),
        &'static str,
    > {
        ensure!(
            CuratorOpeningById::<T, I>::exists(curator_opening_id),
            MSG_CURATOR_OPENING_DOES_NOT_EXIST
        );

        let curator_opening = CuratorOpeningById::<T, I>::get(curator_opening_id);

        let opening = hiring::OpeningById::<T>::get(curator_opening.opening_id);

        Ok((curator_opening, opening))
    }

    fn make_stake_opt_imbalance(
        opt_balance: &Option<BalanceOf<T>>,
        source_account: &T::AccountId,
    ) -> Option<NegativeImbalance<T>> {
        if let Some(balance) = opt_balance {
            let withdraw_result = CurrencyOf::<T>::withdraw(
                source_account,
                *balance,
                WithdrawReasons::all(),
                ExistenceRequirement::AllowDeath,
            );

            assert!(withdraw_result.is_ok());

            withdraw_result.ok()
        } else {
            None
        }
    }

    fn ensure_member_has_no_active_application_on_opening(
        curator_applications: CuratorApplicationIdSet<T>,
        member_id: T::MemberId,
    ) -> Result<(), &'static str> {
        for curator_application_id in curator_applications {
            let curator_application = CuratorApplicationById::<T, I>::get(curator_application_id);
            // Look for application by the member for the opening
            if curator_application.member_id != member_id {
                continue;
            }
            // Get application details
            let application = <hiring::ApplicationById<T>>::get(curator_application.application_id);
            // Return error if application is in active stage
            if application.stage == hiring::ApplicationStage::Active {
                return Err(MSG_MEMBER_HAS_ACTIVE_APPLICATION_ON_OPENING);
            }
        }
        // Member does not have any active applications to the opening
        Ok(())
    }

    fn ensure_curator_application_text_is_valid(text: &[u8]) -> dispatch::Result {
        <CuratorApplicationHumanReadableText<I>>::get().ensure_valid(
            text.len(),
            MSG_CURATOR_APPLICATION_TEXT_TOO_SHORT,
            MSG_CURATOR_APPLICATION_TEXT_TOO_LONG,
        )
    }

    /// CRITICAL:
 /// https://github.com/Joystream/substrate-runtime-joystream/issues/92
 /// This assumes that ensure_can_withdraw can be don
 /// for a sum of balance that later will be actually withdrawn
 /// using individual terms in that sum.
 /// This needs to be fully checked across all possibly scenarios
 /// of actual balance, minimum balance limit, reservation, vesting and locking.
    fn ensure_can_make_stake_imbalance(
        opt_balances: Vec<&Option<BalanceOf<T>>>,
        source_account: &T::AccountId,
    ) -> Result<(), &'static str> {
        let zero_balance = <BalanceOf<T> as Zero>::zero();

        // Total amount to be staked
        let total_amount = opt_balances.iter().fold(zero_balance, |sum, opt_balance| {
            sum + if let Some(balance) = opt_balance {
                *balance
            } else {
                zero_balance
            }
        });

        if total_amount > zero_balance {
            // Ensure that
            if CurrencyOf::<T>::free_balance(source_account) < total_amount {
                Err(MSG_INSUFFICIENT_BALANCE_TO_COVER_STAKE)
            } else {
                let new_balance = CurrencyOf::<T>::free_balance(source_account) - total_amount;

                CurrencyOf::<T>::ensure_can_withdraw(
                    source_account,
                    total_amount,
                    WithdrawReasons::all(),
                    new_balance,
                )
            }
        } else {
            Ok(())
        }
    }

}
