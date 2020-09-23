#![warn(missing_docs)]

use crate::{Instance, Module, Trait};
use frame_support::decl_error;

decl_error! {
    /// Discussion module predefined errors
    pub enum Error for Module<T: Trait<I>, I: Instance>{
        /// Provided stake balance cannot be zero.
        StakeBalanceCannotBeZero,

        /// Opening does not exist.
        OpeningDoesNotExist,

        /// Origin must be controller or root account of member.
        OriginIsNeitherMemberControllerOrRoot,

        /// Cannot fill opening with multiple applications.
        CannotHireMultipleLeaders,

        /// Worker application does not exist.
        WorkerApplicationDoesNotExist,

        /// Working team size limit exceeded.
        MaxActiveWorkerNumberExceeded,

        /// Successful worker application does not exist.
        SuccessfulWorkerApplicationDoesNotExist,

        /// There is leader already, cannot hire another one.
        CannotHireLeaderWhenLeaderExists,

        /// Not a lead account.
        IsNotLeadAccount,

        /// Current lead is not set.
        CurrentLeadNotSet,

        /// Worker does not exist.
        WorkerDoesNotExist,

        /// Invalid origin for a member.
        InvalidMemberOrigin,

        /// Signer is not worker role account.
        SignerIsNotWorkerRoleAccount,

        /// Cannot stake zero.
        CannotStakeZero,

        /// Insufficient balance to cover stake.
        InsufficientBalanceToCoverStake,

        /// Application stake is less than required opening stake.
        ApplicationStakeDoesntMatchOpening,

        /// Origin is not applicant.
        OriginIsNotApplicant,

        /// Invalid operation - worker is leaving.
        WorkerIsLeaving,

        /// Reward could not be zero.
        CannotRewardWithZero,
    }
}
