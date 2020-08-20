#![cfg(test)]

use super::{Error, ReferendumResult, SubStake, SubVote, Trait};
use crate::mock::*;

type Mocks = InstanceMocks<Runtime, Instance0>;
type MockUtils = InstanceMockUtils<Runtime, Instance0>;

/////////////////// Lifetime - referendum start ////////////////////////////////
#[test]
fn referendum_start() {
    MockUtils::origin_access(USER_ADMIN, |origin| {
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(origin, options, winning_target_count, Ok(()));
    });
}

#[test]
fn referendum_start_access_restricted() {
    MockUtils::origin_access(USER_REGULAR, |origin| {
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(
            origin,
            options,
            winning_target_count,
            Err(Error::OriginNotSuperUser),
        );
    });
}

#[test]
fn referendum_start_forbidden_after_start() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let origin = OriginType::Signed(USER_ADMIN);
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Err(Error::ReferendumAlreadyRunning),
        );
    });
}

#[test]
fn referendum_start_no_options() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let origin = OriginType::Signed(USER_ADMIN);
        let options = 0;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options,
            winning_target_count,
            Err(Error::NoReferendumOptions),
        );
    });
}

#[test]
fn referendum_start_too_many_options() {
    let origin = OriginType::Signed(USER_ADMIN);
    let winning_target_count = 1;

    let ok_options = <Runtime as Trait<Instance0>>::MaxReferendumOptions::get();
    let too_many_options: u64 = ok_options + 1;

    let config = default_genesis_config();
    build_test_externalities(config).execute_with(|| {
        Mocks::start_referendum_extrinsic(origin.clone(), ok_options, winning_target_count, Ok(()));
    });

    let config = default_genesis_config();
    build_test_externalities(config).execute_with(|| {
        Mocks::start_referendum_extrinsic(
            origin.clone(),
            too_many_options,
            winning_target_count,
            Err(Error::TooManyReferendumOptions),
        );
    });
}

/////////////////// Lifetime - voting //////////////////////////////////////////
#[test]
fn voting() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let winning_target_count = 1;

        let options = 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));

        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
    });
}

#[test]
fn voting_referendum_not_running() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);

        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Err(Error::ReferendumNotRunning),
        );
    });
}

#[test]
fn voting_voting_stage_overdue() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let winning_target_count = 1;

        let options = 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));

        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Err(Error::ReferendumNotRunning),
        );
    });
}

#[test]
fn voting_stake_too_low() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let winning_target_count = 1;

        let options = 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get() - 1,
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Err(Error::InsufficientStake),
        );
    });
}

#[test]
fn voting_cant_lock_stake() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let winning_target_count = 1;

        let options = 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));

        Runtime::feature_stack_lock(false, true, true);
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution.clone(),
            Err(Error::InsufficientBalanceToStakeCurrency),
        );

        Runtime::feature_stack_lock(true, false, true);
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution.clone(),
            Err(Error::AccountStakeCurrencyFailed),
        );
    });
}

#[test]
fn voting_user_already_voted() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let winning_target_count = 1;

        let options = 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 0,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, _) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution.clone(),
            Ok(()),
        );

        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution.clone(),
            Err(Error::AlreadyVoted),
        );
    });
}

/////////////////// Lifetime - voting finish ///////////////////////////////////

#[test]
fn finish_voting() {
    MockUtils::origin_access(USER_ADMIN, |origin| {
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));

        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();

        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
    });
}

/////////////////// Lifetime - revealing ///////////////////////////////////////

#[test]
fn reveal() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 1,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt,
            vote_distribution,
            Ok(()),
        );
    });
}

#[test]
fn reveal_reveal_stage_not_running() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 1,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );

        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt.clone(),
            vote_distribution.clone(),
            Err(Error::RevealingNotInProgress),
        );

        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        MockUtils::increase_block_number(reveal_stage_duration + 1);
        Mocks::check_revealing_finished(Some(ReferendumResult::NoVotesRevealed));

        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt.clone(),
            vote_distribution.clone(),
            Err(Error::RevealingNotInProgress),
        );
    });
}

#[test]
fn reveal_no_vote() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        Mocks::check_revealing_finished(Some(ReferendumResult::NoVotesRevealed));
    });
}

#[test]
fn reveal_invalid_vote() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let invalid_option = 1000;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: 1,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);
        let invalid_vote_distribution = vec![SubVote {
            option_index: invalid_option,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt,
            invalid_vote_distribution,
            Err(Error::InvalidVote),
        );
    });
}

#[test]
fn reveal_invalid_commitment_proof() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let option_to_vote_for = 0;
        let invalid_option = option_to_vote_for + 1;
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: option_to_vote_for,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let invalid_vote_distribution = vec![SubVote {
            option_index: invalid_option,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt,
            invalid_vote_distribution,
            Err(Error::InvalidReveal),
        );
    });
}

/////////////////// Lifetime - revealing finish ////////////////////////////////

#[test]
fn finish_revealing_period() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let option_to_vote_for = 0;
        let stake = <Runtime as Trait<Instance0>>::MinimumStake::get();
        let stake_distribution = vec![SubStake {
            account_id: account_id,
            balance: stake,
        }];
        let vote_distribution = vec![SubVote {
            option_index: option_to_vote_for,
            balance: stake,
        }];
        let (commitment, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin.clone(),
            account_id,
            account_id,
            commitment,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt,
            vote_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        Mocks::check_revealing_finished(Some(ReferendumResult::Winners(vec![(
            option_to_vote_for,
            1 * stake,
        )])));
    });
}

#[test]
fn finish_revealing_period_no_revealing_stage() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let account_id = USER_ADMIN;
        let origin = OriginType::Signed(account_id);
        let options = 3;
        let winning_target_count = 1;

        let option_to_vote_for = 0;
        let vote_distribution = vec![SubVote {
            option_index: option_to_vote_for,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (_, salt) = MockUtils::calculate_commitment(&account_id, &vote_distribution);

        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt.clone(),
            vote_distribution.clone(),
            Err(Error::RevealingNotInProgress),
        );

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::reveal_vote(
            origin.clone(),
            account_id,
            account_id,
            salt.clone(),
            vote_distribution.clone(),
            Err(Error::RevealingNotInProgress),
        );
    });
}

#[test]
fn finish_revealing_period_vote_power() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_superuser = USER_ADMIN;
        let account_id1 = USER_REGULAR;
        let account_id2 = USER_REGULAR_POWER_VOTES;
        let origin = OriginType::Signed(account_superuser);
        let origin_voter1 = OriginType::Signed(account_id1);
        let origin_voter2 = OriginType::Signed(account_id2);
        let options = 3;
        let winning_target_count = 1;

        let option_to_vote_for1 = 0;
        let option_to_vote_for2 = 1;
        let stake_bigger = <Runtime as Trait<Instance0>>::MinimumStake::get() * 2;
        let stake_smaller = <Runtime as Trait<Instance0>>::MinimumStake::get();
        let stake_distribution1 = vec![SubStake {
            account_id: account_id1,
            balance: stake_bigger,
        }];
        let vote_distribution1 = vec![SubVote {
            option_index: option_to_vote_for1,
            balance: stake_bigger,
        }];
        let stake_distribution2 = vec![SubStake {
            account_id: account_id2,
            balance: stake_smaller,
        }];
        let vote_distribution2 = vec![SubVote {
            option_index: option_to_vote_for2,
            balance: stake_smaller,
        }];
        let (commitment1, salt1) =
            MockUtils::calculate_commitment(&account_id1, &vote_distribution1);
        let (commitment2, salt2) =
            MockUtils::calculate_commitment(&account_id2, &vote_distribution2);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            commitment1,
            stake_distribution1,
            Ok(()),
        ); // vote for first option by regular user
        Mocks::vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            commitment2,
            stake_distribution2,
            Ok(()),
        ); // vote for second option by prominent user
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            salt1,
            vote_distribution1,
            Ok(()),
        );
        Mocks::reveal_vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            salt2,
            vote_distribution2,
            Ok(()),
        );
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        // option 2 should win because prominent user has more powerfull vote with the same stake
        Mocks::check_revealing_finished(Some(ReferendumResult::Winners(vec![(
            option_to_vote_for2,
            1 * stake_smaller * POWER_VOTE_STRENGTH,
        )])));
    });
}

/////////////////// Referendum Winners ////////////////////////////////////////////////////

#[test]
fn winners_no_vote_revealed() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let origin = OriginType::Signed(USER_ADMIN);
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_extrinsic(origin.clone(), options, winning_target_count, Ok(()));
        MockUtils::increase_block_number(voting_stage_duration + 1);
        Mocks::check_voting_finished(options, winning_target_count);
        MockUtils::increase_block_number(reveal_stage_duration + 1);
        Mocks::check_revealing_finished(Some(ReferendumResult::NoVotesRevealed));
    });
}

#[test]
fn winners_multiple_winners() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_superuser = USER_ADMIN;
        let account_id1 = USER_REGULAR;
        let account_id2 = USER_REGULAR_2;
        let account_id3 = USER_REGULAR_3;
        let origin = OriginType::Signed(account_superuser);
        let origin_voter1 = OriginType::Signed(account_id1);
        let origin_voter2 = OriginType::Signed(account_id2);
        let origin_voter3 = OriginType::Signed(account_id3);
        let options = 3;
        let winning_target_count = 2;

        let option_to_vote_for1 = 0;
        let option_to_vote_for2 = 1;
        let stake = <Runtime as Trait<Instance0>>::MinimumStake::get();
        let stake_distribution1 = vec![SubStake {
            account_id: account_id1,
            balance: stake,
        }];
        let vote_distribution1 = vec![SubVote {
            option_index: option_to_vote_for1,
            balance: stake,
        }];
        let stake_distribution2 = vec![SubStake {
            account_id: account_id2,
            balance: stake,
        }];
        let vote_distribution2 = vec![SubVote {
            option_index: option_to_vote_for1,
            balance: stake,
        }];
        let stake_distribution3 = vec![SubStake {
            account_id: account_id3,
            balance: stake,
        }];
        let vote_distribution3 = vec![SubVote {
            option_index: option_to_vote_for2,
            balance: stake,
        }];
        let (commitment1, salt1) =
            MockUtils::calculate_commitment(&account_id1, &vote_distribution1);
        let (commitment2, salt2) =
            MockUtils::calculate_commitment(&account_id2, &vote_distribution2);
        let (commitment3, salt3) =
            MockUtils::calculate_commitment(&account_id3, &vote_distribution3);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            commitment1,
            stake_distribution1,
            Ok(()),
        );
        Mocks::vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            commitment2,
            stake_distribution2,
            Ok(()),
        );
        Mocks::vote(
            origin_voter3.clone(),
            account_id3,
            account_id3,
            commitment3,
            stake_distribution3,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            salt1,
            vote_distribution1,
            Ok(()),
        );
        Mocks::reveal_vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            salt2,
            vote_distribution2,
            Ok(()),
        );
        Mocks::reveal_vote(
            origin_voter3.clone(),
            account_id3,
            account_id3,
            salt3,
            vote_distribution3,
            Ok(()),
        );
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        let expected_winners = vec![
            (option_to_vote_for1, 2 * stake),
            (option_to_vote_for2, 1 * stake),
        ];

        Mocks::check_revealing_finished(Some(ReferendumResult::Winners(expected_winners)));
    });
}

#[test]
// option 1 and option 2 has the same amount of votes but only 1 winner is expected
fn winners_multiple_winners_extra() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_superuser = USER_ADMIN;
        let account_id1 = USER_REGULAR;
        let account_id2 = USER_REGULAR_2;
        let origin = OriginType::Signed(account_superuser);
        let origin_voter1 = OriginType::Signed(account_id1);
        let origin_voter2 = OriginType::Signed(account_id2);
        let options = 3;
        let winning_target_count = 1;

        let option_to_vote_for1 = 0;
        let option_to_vote_for2 = 1;
        let stake = <Runtime as Trait<Instance0>>::MinimumStake::get();
        let stake_distribution1 = vec![SubStake {
            account_id: account_id1,
            balance: stake,
        }];
        let vote_distribution1 = vec![SubVote {
            option_index: option_to_vote_for1,
            balance: stake,
        }];
        let stake_distribution2 = vec![SubStake {
            account_id: account_id2,
            balance: stake,
        }];
        let vote_distribution2 = vec![SubVote {
            option_index: option_to_vote_for2,
            balance: stake,
        }];
        let (commitment1, salt1) =
            MockUtils::calculate_commitment(&account_id1, &vote_distribution1);
        let (commitment2, salt2) =
            MockUtils::calculate_commitment(&account_id2, &vote_distribution2);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            commitment1,
            stake_distribution1,
            Ok(()),
        );
        Mocks::vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            commitment2,
            stake_distribution2,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            salt1,
            vote_distribution1,
            Ok(()),
        );
        Mocks::reveal_vote(
            origin_voter2.clone(),
            account_id2,
            account_id2,
            salt2,
            vote_distribution2,
            Ok(()),
        );
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        let expected_winners = vec![
            (option_to_vote_for1, 1 * stake),
            (option_to_vote_for2, 1 * stake),
        ];

        Mocks::check_revealing_finished(Some(ReferendumResult::ExtraWinners(expected_winners)));
    });
}

#[test]
fn winners_multiple_not_enough() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let voting_stage_duration = <Runtime as Trait<Instance0>>::VoteStageDuration::get();
        let reveal_stage_duration = <Runtime as Trait<Instance0>>::RevealStageDuration::get();
        let account_superuser = USER_ADMIN;
        let account_id1 = USER_REGULAR;
        let origin = OriginType::Signed(account_superuser);
        let origin_voter1 = OriginType::Signed(account_id1);
        let options = 3;
        let winning_target_count = 2;

        let option_to_vote_for = 0;
        let stake = <Runtime as Trait<Instance0>>::MinimumStake::get();
        let stake_distribution = vec![SubStake {
            account_id: account_id1,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let vote_distribution = vec![SubVote {
            option_index: option_to_vote_for,
            balance: <Runtime as Trait<Instance0>>::MinimumStake::get(),
        }];
        let (commitment1, salt1) =
            MockUtils::calculate_commitment(&account_id1, &vote_distribution);

        Mocks::start_referendum_extrinsic(
            origin.clone(),
            options.clone(),
            winning_target_count,
            Ok(()),
        );
        Mocks::vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            commitment1,
            stake_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(voting_stage_duration + 1);

        Mocks::check_voting_finished(options, winning_target_count);
        Mocks::reveal_vote(
            origin_voter1.clone(),
            account_id1,
            account_id1,
            salt1,
            vote_distribution,
            Ok(()),
        );
        MockUtils::increase_block_number(reveal_stage_duration + 1);

        let expected_winners = vec![(option_to_vote_for, 1 * stake)];

        Mocks::check_revealing_finished(Some(ReferendumResult::NotEnoughWinners(expected_winners)));
    });
}

/////////////////// ReferendumManager //////////////////////////////////////////
#[test]
fn referendum_manager_referendum_start() {
    let config = default_genesis_config();

    build_test_externalities(config).execute_with(|| {
        let options = 1;
        let winning_target_count = 1;

        Mocks::start_referendum_manager(options, winning_target_count, Ok(()));
    });
}
