#![cfg(feature = "runtime-benchmarks")]
use super::*;
use crate::Module as ProposalsDiscussion;
use core::convert::TryInto;
use frame_benchmarking::{account, benchmarks};
use membership::Module as Membership;
use sp_std::prelude::*;
use system as frame_system;
use system::EventRecord;
use system::Module as System;
use system::RawOrigin;

const SEED: u32 = 0;

fn get_byte(num: u32, byte_number: u8) -> u8 {
    ((num & (0xff << (8 * byte_number))) >> 8 * byte_number) as u8
}

// Method to generate a distintic valid handle
// for a membership. For each index.
// TODO: This will only work as long as max_handle_length >= 4
fn handle_from_id<T: membership::Trait>(id: u32) -> Vec<u8> {
    let min_handle_length = Membership::<T>::min_handle_length();
    // If the index is ever different from u32 change this
    let mut handle = vec![
        get_byte(id, 0),
        get_byte(id, 1),
        get_byte(id, 2),
        get_byte(id, 3),
    ];

    while handle.len() < (min_handle_length as usize) {
        handle.push(0u8);
    }

    handle
}

fn assert_last_event<T: Trait>(generic_event: <T as Trait>::Event) {
    let events = System::<T>::events();
    let system_event: <T as frame_system::Trait>::Event = generic_event.into();
    // compare to the last event record
    let EventRecord { event, .. } = &events[events.len() - 1];
    assert_eq!(event, &system_event);
}

fn member_account<T: membership::Trait>(
    name: &'static str,
    id: u32,
) -> (T::AccountId, T::MemberId) {
    let account_id = account::<T::AccountId>(name, id, SEED);
    let handle = handle_from_id::<T>(id);

    let authority_account = account::<T::AccountId>(name, 0, SEED);

    Membership::<T>::set_screening_authority(RawOrigin::Root.into(), authority_account.clone())
        .unwrap();

    Membership::<T>::add_screened_member(
        RawOrigin::Signed(authority_account.clone()).into(),
        account_id.clone(),
        Some(handle),
        None,
        None,
    )
    .unwrap();

    (account_id, T::MemberId::from(id.try_into().unwrap()))
}

benchmarks! {
    _ { }

    add_post {
        let i in 1 .. T::MaxWhiteListSize::get();

        // TODO: this parameter doesn't affect the running time
        // maybe we should bound it here with the UI limit?
        let j in 0 .. 50000;

        // We do this to ignore the id 0 because the `Test` runtime
        // returns 0 as an invalid id but 0 as a valid one
        let (_, _) = member_account::<T>("member", 0);
        let (account_id, caller_member_id) = member_account::<T>("caller_member", 1);

        let mut whitelisted_members = vec![caller_member_id];

        for id in 2 .. i + 1 {
            let (_, member_id) = member_account::<T>("member", id);
            whitelisted_members.push(member_id);
        }

        let thread_id = ProposalsDiscussion::<T>::create_thread(
            caller_member_id,
            ThreadMode::Closed(whitelisted_members)
        ).unwrap();

        assert!(ThreadById::<T>::contains_key(thread_id), "Thread not created");

        let text = vec![0u8; j.try_into().unwrap()];

    }: _ (RawOrigin::Signed(account_id), caller_member_id, thread_id, text)
    verify {
        let post_id = T::PostId::from(1);

        assert!(PostThreadIdByPostId::<T>::contains_key(thread_id, post_id), "Post not created");
        assert_eq!(
            PostThreadIdByPostId::<T>::get(thread_id, post_id).author_id,
            caller_member_id,
            "Post author isn't correct"
        );

        assert_last_event::<T>(RawEvent::PostCreated(post_id, caller_member_id).into());
    }

    update_post {
        // TODO: this parameter doesn't affect the running time
        // maybe we should bound it here with the UI limit?
        let j in 0 .. 50000;

        // We do this to ignore the id 0 because the `Test` runtime
        // returns 0 as an invalid id but 0 as a valid one
        let (_, _) = member_account::<T>("caller_member", 0);
        let (account_id, caller_member_id) = member_account::<T>("caller_member", 1);

        let thread_id = ProposalsDiscussion::<T>::create_thread(
            caller_member_id,
            ThreadMode::Open
        ).unwrap();

        assert!(ThreadById::<T>::contains_key(thread_id), "Thread not created");

        ProposalsDiscussion::<T>::add_post(
            RawOrigin::Signed(account_id.clone()).into(),
            caller_member_id,
            thread_id,
            vec![0u8]
        ).unwrap();

        let post_id = T::PostId::from(1);

        assert!(PostThreadIdByPostId::<T>::contains_key(thread_id, post_id), "Post not created");

        let new_text = vec![0u8; j.try_into().unwrap()];
    }: _ (RawOrigin::Signed(account_id), caller_member_id, thread_id, post_id, new_text)
    verify {
        assert_last_event::<T>(RawEvent::PostUpdated(post_id, caller_member_id).into());
    }

    // TODO: this extrinsic uses `T::CouncilOriginValidator::ensure_actor_origin`
    // this is a hook to the runtime. Since the pallet implementation shouldn't have any
    // information on the runtime this hooks should be constant.
    // However, the implementation in the runtime is linear in the number of council members.
    // Even if we use that knowledge we need to create an artificial dependency with `governance`
    // to be able to correctly benchmark this.
    change_thread_mode {
        let i in 1 .. T::MaxWhiteListSize::get();

        // We do this to ignore the id 0 because the `Test` runtime
        // returns 0 as an invalid id but 0 as a valid one
        let (_, _) = member_account::<T>("member", 0);
        let (account_id, caller_member_id) = member_account::<T>("caller_member", 1);


        let thread_id = ProposalsDiscussion::<T>::create_thread(
            caller_member_id,
            ThreadMode::Open
        ).unwrap();

        assert!(ThreadById::<T>::contains_key(thread_id), "Thread not created");


        let mut whitelisted_members = vec![caller_member_id];

        for id in 2 .. i + 1 {
            let (_, member_id) = member_account::<T>("member", id);
            whitelisted_members.push(member_id);
        }

        let mode = ThreadMode::Closed(whitelisted_members);
    }: _ (RawOrigin::Signed(account_id), caller_member_id, thread_id, mode.clone())
    verify {
        assert_eq!(
            ProposalsDiscussion::<T>::thread_by_id(thread_id).mode,
            mode.clone(),
            "Thread not correctly updated"
        );

        assert_last_event::<T>(RawEvent::ThreadModeChanged(thread_id, mode).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{initial_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_add_post() {
        initial_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_add_post::<Test>());
        });
    }

    #[test]
    fn test_update_post() {
        initial_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_update_post::<Test>());
        });
    }

    #[test]
    fn test_change_thread_mode() {
        initial_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_change_thread_mode::<Test>());
        });
    }
}
