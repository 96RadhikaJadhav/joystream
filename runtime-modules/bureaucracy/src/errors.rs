use srml_support::decl_error;

use membership::members;

decl_error! {
    /// Discussion module predefined errors
    pub enum Error {
        /// Current lead is not set.
        CurrentLeadNotSet,

        /// Not a lead account.
        IsNotLeadAccount,

        /// Opening text too short.
        OpeningTextTooShort,

        /// Opening text too long.
        OpeningTextTooLong,

        /// Worker opening does not exist.
        WorkerOpeningDoesNotExist,

        /// Insufficient balance to apply.
        InsufficientBalanceToApply,

        /// Unsigned origin.
        MembershipUnsignedOrigin,

        /// Member id is invalid.
        MembershipInvalidMemberId,

        /// Signer does not match controller account.
        ApplyOnWorkerOpeningSignerNotControllerAccount,

        /// Origin must be controller or root account of member.
        OriginIsNeitherMemberControllerOrRoot,

        /// Member already has an active application on the opening.
        MemberHasActiveApplicationOnOpening,

        /// Worker application text too long.
        WorkerApplicationTextTooLong,

        /// Worker application text too short.
        WorkerApplicationTextTooShort,

        /// Insufficient balance to cover stake.
        InsufficientBalanceToCoverStake,

        /// Origin is not applicant.
        OriginIsNotApplicant,

        /// Worker application does not exist.
        WorkerApplicationDoesNotExist,

        /// Successful worker application does not exist.
        SuccessfulWorkerApplicationDoesNotExist,

        /// Reward policy has invalid next payment block number.
        FillWorkerOpeningInvalidNextPaymentBlock,

        /// Working group mint does not exist.
        FillWorkerOpeningMintDoesNotExist,

        ///Relationship must exist.
        RelationshipMustExist,

        /// Worker exit rationale text is too long.
        WorkerExitRationaleTextTooLong,

        /// Worker exit rationale text is too short.
        WorkerExitRationaleTextTooShort,

        /// Unstaker does not exist.
        UnstakerDoesNotExist,

        /// Only workers can unstake.
        OnlyWorkersCanUnstake,

        /// Worker must be in unstaking stage.
        WorkerIsNotUnstaking,

        /// Signer is not worker role account.
        SignerIsNotWorkerRoleAccount,

        /// Worker has no recurring reward.
        WorkerHasNoReward,

        /// Worker is not active.
        WorkerIsNotActive,

        /// Worker does not exist.
        WorkerDoesNotExist,
    }
}

impl From<system::Error> for Error {
    fn from(error: system::Error) -> Self {
        match error {
            system::Error::Other(msg) => Error::Other(msg),
            _ => Error::Other(error.into()),
        }
    }
}

/*
 * The errors below, while in many cases encoding similar outcomes,
 * are scoped to the specific extrinsic for which they are used.
 * The reason for this is that it will later to easier to convert this
 * representation into into the type safe error encoding coming in
 * later versions of Substrate.
 */

// Errors for `accept_worker_applications`
pub static MSG_ACCEPT_WORKER_APPLICATIONS_OPENING_DOES_NOT_EXIST: &str = "Opening does not exist";
pub static MSG_ACCEPT_WORKER_APPLICATIONS_OPENING_IS_NOT_WAITING_TO_BEGIN: &str =
    "Opening Is Not in Waiting to begin";

// Errors for `begin_worker_applicant_review`
pub static MSG_BEGIN_WORKER_APPLICANT_REVIEW_OPENING_DOES_NOT_EXIST: &str =
    "Opening does not exist";
pub static MSG_BEGIN_WORKER_APPLICANT_REVIEW_OPENING_OPENING_IS_NOT_WAITING_TO_BEGIN: &str =
    "Opening Is Not in Waiting";

// Errors for `fill_worker_opening`
pub static MSG_FULL_WORKER_OPENING_OPENING_DOES_NOT_EXIST: &str = "OpeningDoesNotExist";
pub static MSG_FULL_WORKER_OPENING_OPENING_NOT_IN_REVIEW_PERIOD_STAGE: &str =
    "OpeningNotInReviewPeriodStage";
pub static MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_TOO_SHORT: &str =
    "Application stake unstaking period for successful applicants too short";
pub static MSG_FULL_WORKER_OPENING_SUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_TOO_SHORT: &str =
    "Application stake unstaking period for failed applicants too short";
pub static MSG_FULL_WORKER_OPENING_SUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_TOO_SHORT: &str =
    "Role stake unstaking period for successful applicants too short";
pub static MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_TOO_SHORT: &str =
    "Role stake unstaking period for failed applicants too short";
pub static MSG_FULL_WORKER_OPENING_SUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_REDUNDANT: &str =
    "Application stake unstaking period for successful applicants redundant";
pub static MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_REDUNDANT: &str =
    "Application stake unstaking period for failed applicants redundant";
pub static MSG_FULL_WORKER_OPENING_SUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_REDUNDANT: &str =
    "Role stake unstaking period for successful applicants redundant";
pub static MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_REDUNDANT: &str =
    "Role stake unstaking period for failed applicants redundant";
pub static MSG_FULL_WORKER_OPENING_APPLICATION_DOES_NOT_EXIST: &str = "ApplicationDoesNotExist";
pub static MSG_FULL_WORKER_OPENING_APPLICATION_NOT_ACTIVE: &str = "ApplicationNotInActiveStage";
pub static MSG_FILL_WORKER_OPENING_APPLICATION_FOR_WRONG_OPENING: &str =
    "Applications not for opening";
// Errors for `withdraw_worker_application`
pub static MSG_WITHDRAW_WORKER_APPLICATION_APPLICATION_DOES_NOT_EXIST: &str =
    "ApplicationDoesNotExist";
pub static MSG_WITHDRAW_WORKER_APPLICATION_APPLICATION_NOT_ACTIVE: &str = "ApplicationNotActive";
pub static MSG_WITHDRAW_WORKER_APPLICATION_OPENING_NOT_ACCEPTING_APPLICATIONS: &str =
    "OpeningNotAcceptingApplications";
pub static MSG_WITHDRAW_WORKER_APPLICATION_UNSTAKING_PERIOD_TOO_SHORT: &str =
    "UnstakingPeriodTooShort ..."; // <== SHOULD REALLY BE TWO SEPARATE, ONE FOR EACH STAKING PURPOSE
pub static MSG_WITHDRAW_WORKER_APPLICATION_REDUNDANT_UNSTAKING_PERIOD: &str =
    "RedundantUnstakingPeriodProvided ...";

// Errors for `create_channel`
pub static MSG_CREATE_CHANNEL_IS_NOT_MEMBER: &str = "Is not a member";
pub static MSG_CREATE_CHANNEL_NOT_CONTROLLER_ACCOUNT: &str =
    "Account is not controller account of member";

// Errors for `add_worker_opening`
pub static MSG_ADD_WORKER_OPENING_ACTIVATES_IN_THE_PAST: &str =
    "Opening does not activate in the future";
pub static MSG_ADD_WORKER_OPENING_ROLE_STAKE_LESS_THAN_MINIMUM: &str =
    "Role stake amount less than minimum currency balance";
pub static MSG_ADD_WORKER_OPENING_APPLIICATION_STAKE_LESS_THAN_MINIMUM: &str =
    "Application stake amount less than minimum currency balance";
pub static MSG_ADD_WORKER_OPENING_OPENING_DOES_NOT_EXIST: &str = "OpeningDoesNotExist";
pub static MSG_ADD_WORKER_OPENING_STAKE_PROVIDED_WHEN_REDUNDANT: &str =
    "StakeProvidedWhenRedundant ..."; // <== SHOULD REALLY BE TWO SEPARATE, ONE FOR EACH STAKING PURPOSE
pub static MSG_ADD_WORKER_OPENING_STAKE_MISSING_WHEN_REQUIRED: &str =
    "StakeMissingWhenRequired ..."; // <== SHOULD REALLY BE TWO SEPARATE, ONE FOR EACH STAKING PURPOSE
pub static MSG_ADD_WORKER_OPENING_STAKE_AMOUNT_TOO_LOW: &str = "StakeAmountTooLow ..."; // <== SHOULD REALLY BE TWO SEPARATE, ONE FOR EACH STAKING PURPOSE
pub static MSG_ADD_WORKER_OPENING_OPENING_NOT_IN_ACCEPTING_APPLICATION_STAGE: &str =
    "OpeningNotInAcceptingApplicationsStage";
pub static MSG_ADD_WORKER_OPENING_NEW_APPLICATION_WAS_CROWDED_OUT: &str =
    "NewApplicationWasCrowdedOut";
pub static MSG_ADD_WORKER_OPENING_ZERO_MAX_APPLICANT_COUNT: &str =
    "Application rationing has zero max active applicants";
pub static MSG_RECURRING_REWARDS_NEXT_PAYMENT_NOT_IN_FUTURE: &str =
    "Next payment is not in the future";
pub static MSG_RECURRING_REWARDS_RECIPIENT_NOT_FOUND: &str = "Recipient not found";
pub static MSG_RECURRING_REWARDS_REWARD_SOURCE_NOT_FOUND: &str =
    "Recipient reward source not found";
pub static MSG_RECURRING_REWARDS_REWARD_RELATIONSHIP_NOT_FOUND: &str =
    "Reward relationship not found";
pub static MSG_STAKING_ERROR_STAKE_NOT_FOUND: &str = "Stake not found";
pub static MSG_STAKING_ERROR_UNSTAKING_PERIOD_SHOULD_BE_GREATER_THAN_ZERO: &str =
    "Unstaking period should be greater than zero";
pub static MSG_STAKING_ERROR_ALREADY_UNSTAKING: &str = "Already unstaking";
pub static MSG_STAKING_ERROR_NOT_STAKED: &str = "Not staked";
pub static MSG_STAKING_ERROR_CANNOT_UNSTAKE_WHILE_SLASHES_ONGOING: &str =
    "Cannot unstake while slashes ongoing";
pub static MSG_MEMBERSHIP_UNSIGNED_ORIGIN: &str = "Unsigned origin";
pub static MSG_MEMBERSHIP_INVALID_MEMBER_ID: &str = "Member id is invalid";
pub static MSG_APPLY_ON_WORKER_OPENING_SIGNER_NOT_CONTROLLER_ACCOUNT: &str =
    "Signer does not match controller account";

/// Error wrapper for external module error conversions.
pub struct WrappedError<E> {
    /// Generic error.
    pub error: E,
}

/// Helps with conversion of other modules errors.
#[macro_export]
macro_rules! ensure_on_wrapped_error {
    ($call:expr) => {{
        { $call }
            .map_err(|err| crate::WrappedError { error: err })
            .map_err(|err| Error::Other(err.into()))
    }};
}

impl rstd::convert::From<WrappedError<hiring::BeginAcceptingApplicationsError>> for &str {
    fn from(wrapper: WrappedError<hiring::BeginAcceptingApplicationsError>) -> Self {
        match wrapper.error {
            hiring::BeginAcceptingApplicationsError::OpeningDoesNotExist => {
                MSG_ACCEPT_WORKER_APPLICATIONS_OPENING_DOES_NOT_EXIST
            }
            hiring::BeginAcceptingApplicationsError::OpeningIsNotInWaitingToBeginStage => {
                MSG_ACCEPT_WORKER_APPLICATIONS_OPENING_IS_NOT_WAITING_TO_BEGIN
            }
        }
    }
}

impl rstd::convert::From<WrappedError<hiring::AddOpeningError>> for &str {
    fn from(wrapper: WrappedError<hiring::AddOpeningError>) -> Self {
        match wrapper.error {
            hiring::AddOpeningError::OpeningMustActivateInTheFuture => {
                MSG_ADD_WORKER_OPENING_ACTIVATES_IN_THE_PAST
            }
            hiring::AddOpeningError::StakeAmountLessThanMinimumCurrencyBalance(purpose) => {
                match purpose {
                    hiring::StakePurpose::Role => {
                        MSG_ADD_WORKER_OPENING_ROLE_STAKE_LESS_THAN_MINIMUM
                    }
                    hiring::StakePurpose::Application => {
                        MSG_ADD_WORKER_OPENING_APPLIICATION_STAKE_LESS_THAN_MINIMUM
                    }
                }
            }
            hiring::AddOpeningError::ApplicationRationingZeroMaxApplicants => {
                MSG_ADD_WORKER_OPENING_ZERO_MAX_APPLICANT_COUNT
            }
        }
    }
}

impl rstd::convert::From<WrappedError<hiring::BeginReviewError>> for &str {
    fn from(wrapper: WrappedError<hiring::BeginReviewError>) -> Self {
        match wrapper.error {
            hiring::BeginReviewError::OpeningDoesNotExist => {
                MSG_BEGIN_WORKER_APPLICANT_REVIEW_OPENING_DOES_NOT_EXIST
            }
            hiring::BeginReviewError::OpeningNotInAcceptingApplicationsStage => {
                MSG_BEGIN_WORKER_APPLICANT_REVIEW_OPENING_OPENING_IS_NOT_WAITING_TO_BEGIN
            }
        }
    }
}

impl<T: hiring::Trait> rstd::convert::From<WrappedError<hiring::FillOpeningError<T>>> for &str {
    fn from(wrapper: WrappedError<hiring::FillOpeningError<T>>) -> Self {
        match wrapper.error {
			hiring::FillOpeningError::<T>::OpeningDoesNotExist => MSG_FULL_WORKER_OPENING_OPENING_DOES_NOT_EXIST,
			hiring::FillOpeningError::<T>::OpeningNotInReviewPeriodStage => MSG_FULL_WORKER_OPENING_OPENING_NOT_IN_REVIEW_PERIOD_STAGE,
			hiring::FillOpeningError::<T>::UnstakingPeriodTooShort(
				stake_purpose,
				outcome_in_filled_opening,
			) => match stake_purpose {
				hiring::StakePurpose::Application => match outcome_in_filled_opening {
					hiring::ApplicationOutcomeInFilledOpening::Success => MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_TOO_SHORT,
					hiring::ApplicationOutcomeInFilledOpening::Failure => MSG_FULL_WORKER_OPENING_SUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_TOO_SHORT
				},
				hiring::StakePurpose::Role => match outcome_in_filled_opening {
					hiring::ApplicationOutcomeInFilledOpening::Success => MSG_FULL_WORKER_OPENING_SUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_TOO_SHORT,
					hiring::ApplicationOutcomeInFilledOpening::Failure => MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_TOO_SHORT
				},
			},
			hiring::FillOpeningError::<T>::RedundantUnstakingPeriodProvided(
				stake_purpose,
				outcome_in_filled_opening,
			) => match stake_purpose {
				hiring::StakePurpose::Application => match outcome_in_filled_opening {
					hiring::ApplicationOutcomeInFilledOpening::Success => MSG_FULL_WORKER_OPENING_SUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_REDUNDANT,
					hiring::ApplicationOutcomeInFilledOpening::Failure => MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_APPLICATION_STAKE_UNSTAKING_PERIOD_REDUNDANT
				},
				hiring::StakePurpose::Role => match outcome_in_filled_opening {
					hiring::ApplicationOutcomeInFilledOpening::Success => MSG_FULL_WORKER_OPENING_SUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_REDUNDANT,
					hiring::ApplicationOutcomeInFilledOpening::Failure => MSG_FULL_WORKER_OPENING_UNSUCCESSFUL_ROLE_STAKE_UNSTAKING_PERIOD_REDUNDANT
				},
			},
			hiring::FillOpeningError::<T>::ApplicationDoesNotExist(_application_id) => MSG_FULL_WORKER_OPENING_APPLICATION_DOES_NOT_EXIST,
			hiring::FillOpeningError::<T>::ApplicationNotInActiveStage(_application_id) => MSG_FULL_WORKER_OPENING_APPLICATION_NOT_ACTIVE,
			hiring::FillOpeningError::<T>::ApplicationForWrongOpening(_application_id) => MSG_FILL_WORKER_OPENING_APPLICATION_FOR_WRONG_OPENING,
		}
    }
}

impl rstd::convert::From<WrappedError<hiring::DeactivateApplicationError>> for &str {
    fn from(wrapper: WrappedError<hiring::DeactivateApplicationError>) -> Self {
        match wrapper.error {
            hiring::DeactivateApplicationError::ApplicationDoesNotExist => {
                MSG_WITHDRAW_WORKER_APPLICATION_APPLICATION_DOES_NOT_EXIST
            }
            hiring::DeactivateApplicationError::ApplicationNotActive => {
                MSG_WITHDRAW_WORKER_APPLICATION_APPLICATION_NOT_ACTIVE
            }
            hiring::DeactivateApplicationError::OpeningNotAcceptingApplications => {
                MSG_WITHDRAW_WORKER_APPLICATION_OPENING_NOT_ACCEPTING_APPLICATIONS
            }
            hiring::DeactivateApplicationError::UnstakingPeriodTooShort(_stake_purpose) => {
                MSG_WITHDRAW_WORKER_APPLICATION_UNSTAKING_PERIOD_TOO_SHORT
            }
            hiring::DeactivateApplicationError::RedundantUnstakingPeriodProvided(
                _stake_purpose,
            ) => MSG_WITHDRAW_WORKER_APPLICATION_REDUNDANT_UNSTAKING_PERIOD,
        }
    }
}

impl rstd::convert::From<WrappedError<members::ControllerAccountForMemberCheckFailed>> for &str {
    fn from(wrapper: WrappedError<members::ControllerAccountForMemberCheckFailed>) -> Self {
        match wrapper.error {
            members::ControllerAccountForMemberCheckFailed::NotMember => {
                MSG_CREATE_CHANNEL_IS_NOT_MEMBER
            }
            members::ControllerAccountForMemberCheckFailed::NotControllerAccount => {
                MSG_CREATE_CHANNEL_NOT_CONTROLLER_ACCOUNT
            }
        }
    }
}

impl rstd::convert::From<WrappedError<hiring::AddApplicationError>> for &str {
    fn from(wrapper: WrappedError<hiring::AddApplicationError>) -> Self {
        match wrapper.error {
            hiring::AddApplicationError::OpeningDoesNotExist => {
                MSG_ADD_WORKER_OPENING_OPENING_DOES_NOT_EXIST
            }
            hiring::AddApplicationError::StakeProvidedWhenRedundant(_stake_purpose) => {
                MSG_ADD_WORKER_OPENING_STAKE_PROVIDED_WHEN_REDUNDANT
            }
            hiring::AddApplicationError::StakeMissingWhenRequired(_stake_purpose) => {
                MSG_ADD_WORKER_OPENING_STAKE_MISSING_WHEN_REQUIRED
            }
            hiring::AddApplicationError::StakeAmountTooLow(_stake_purpose) => {
                MSG_ADD_WORKER_OPENING_STAKE_AMOUNT_TOO_LOW
            }
            hiring::AddApplicationError::OpeningNotInAcceptingApplicationsStage => {
                MSG_ADD_WORKER_OPENING_OPENING_NOT_IN_ACCEPTING_APPLICATION_STAGE
            }
            hiring::AddApplicationError::NewApplicationWasCrowdedOut => {
                MSG_ADD_WORKER_OPENING_NEW_APPLICATION_WAS_CROWDED_OUT
            }
        }
    }
}

impl rstd::convert::From<WrappedError<members::MemberControllerAccountDidNotSign>> for &str {
    fn from(wrapper: WrappedError<members::MemberControllerAccountDidNotSign>) -> Self {
        match wrapper.error {
            members::MemberControllerAccountDidNotSign::UnsignedOrigin => {
                MSG_MEMBERSHIP_UNSIGNED_ORIGIN
            }
            members::MemberControllerAccountDidNotSign::MemberIdInvalid => {
                MSG_MEMBERSHIP_INVALID_MEMBER_ID
            }
            members::MemberControllerAccountDidNotSign::SignerControllerAccountMismatch => {
                MSG_APPLY_ON_WORKER_OPENING_SIGNER_NOT_CONTROLLER_ACCOUNT
            }
        }
    }
}

impl rstd::convert::From<WrappedError<recurringrewards::RewardsError>> for &str {
    fn from(wrapper: WrappedError<recurringrewards::RewardsError>) -> Self {
        match wrapper.error {
            recurringrewards::RewardsError::NextPaymentNotInFuture => {
                MSG_RECURRING_REWARDS_NEXT_PAYMENT_NOT_IN_FUTURE
            }
            recurringrewards::RewardsError::RecipientNotFound => {
                MSG_RECURRING_REWARDS_RECIPIENT_NOT_FOUND
            }
            recurringrewards::RewardsError::RewardSourceNotFound => {
                MSG_RECURRING_REWARDS_REWARD_SOURCE_NOT_FOUND
            }
            recurringrewards::RewardsError::RewardRelationshipNotFound => {
                MSG_RECURRING_REWARDS_REWARD_RELATIONSHIP_NOT_FOUND
            }
        }
    }
}

impl rstd::convert::From<WrappedError<stake::StakeActionError<stake::InitiateUnstakingError>>>
    for &str
{
    fn from(wrapper: WrappedError<stake::StakeActionError<stake::InitiateUnstakingError>>) -> Self {
        match wrapper.error {
            stake::StakeActionError::StakeNotFound => MSG_STAKING_ERROR_STAKE_NOT_FOUND,
            stake::StakeActionError::Error(initiate_unstaking_error) => {
                match initiate_unstaking_error {
                    stake::InitiateUnstakingError::UnstakingPeriodShouldBeGreaterThanZero => {
                        MSG_STAKING_ERROR_UNSTAKING_PERIOD_SHOULD_BE_GREATER_THAN_ZERO
                    }
                    stake::InitiateUnstakingError::UnstakingError(unstaking_error) => {
                        match unstaking_error {
                            stake::UnstakingError::AlreadyUnstaking => {
                                MSG_STAKING_ERROR_ALREADY_UNSTAKING
                            }
                            stake::UnstakingError::NotStaked => MSG_STAKING_ERROR_NOT_STAKED,
                            stake::UnstakingError::CannotUnstakeWhileSlashesOngoing => {
                                MSG_STAKING_ERROR_CANNOT_UNSTAKE_WHILE_SLASHES_ONGOING
                            }
                        }
                    }
                }
            }
        }
    }
}
