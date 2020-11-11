use crate::{BalanceOf, Module, ProposalParameters};

// Proposal parameters for the 'Terminate working group leader role' proposal
pub(crate) fn terminate_working_group_leader_role_proposal<T: crate::Trait>(
) -> ProposalParameters<T::BlockNumber, BalanceOf<T>> {
    ProposalParameters {
        voting_period: <Module<T>>::terminate_working_group_leader_role_proposal_voting_period(),
        grace_period: <Module<T>>::terminate_working_group_leader_role_proposal_grace_period(),
        approval_quorum_percentage: 66,
        approval_threshold_percentage: 80,
        slashing_quorum_percentage: 60,
        slashing_threshold_percentage: 80,
        required_stake: Some(<BalanceOf<T>>::from(100_000_u32)),
        constitutionality: 1,
    }
}

// Proposal parameters for the 'Amend constitution' proposal
pub(crate) fn amend_constitution_proposal<T: crate::Trait>(
) -> ProposalParameters<T::BlockNumber, BalanceOf<T>> {
    ProposalParameters {
        voting_period: <Module<T>>::amend_constitution_proposal_voting_period(),
        grace_period: <Module<T>>::amend_constitution_proposal_grace_period(),
        approval_quorum_percentage: 80,
        approval_threshold_percentage: 100,
        slashing_quorum_percentage: 60,
        slashing_threshold_percentage: 80,
        required_stake: Some(<BalanceOf<T>>::from(1_000_000_u32)),
        constitutionality: 1,
    }
}
