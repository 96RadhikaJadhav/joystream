//! Proposals discussion module for the Joystream platform. Version 2.
//! Contains discussion subsystem for the proposals engine.
//!
//! Supported extrinsics:
//! - add_post - adds a post to existing discussion thread
//! - update_post - updates existing post
//!
//! Public API:
//! - create_discussion - creates a discussion
//!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Do not delete! Cannot be uncommented by default, because of Parity decl_module! issue.
//#![warn(missing_docs)]

mod errors;
#[cfg(test)]
mod tests;
mod types;

use rstd::clone::Clone;
use rstd::prelude::*;
use rstd::vec::Vec;
use srml_support::{decl_event, decl_module, decl_storage, ensure, Parameter};

use srml_support::traits::Get;
pub use types::{ActorOriginValidator, ThreadPostActorOriginValidator};
use types::{Post, Thread, ThreadCounter};

pub(crate) use types::MemberId;

// TODO: move errors to decl_error macro (after substrate version upgrade)

decl_event!(
    /// Proposals engine events
    pub enum Event<T>
    where
        <T as Trait>::ThreadId,
        MemberId = MemberId<T>,
        <T as Trait>::PostId,
        <T as Trait>::PostAuthorId,
    {
    	/// Emits on thread creation.
        ThreadCreated(ThreadId, MemberId),

    	/// Emits on post creation.
        PostCreated(PostId, PostAuthorId),

    	/// Emits on post update.
        PostUpdated(PostId, PostAuthorId),
    }
);

/// 'Proposal discussion' substrate module Trait
pub trait Trait: system::Trait + membership::members::Trait {
    /// Engine event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// Validates thread author id and origin combination
    type ThreadAuthorOriginValidator: ActorOriginValidator<Self::Origin, MemberId<Self>>;

    /// Validates post author id and origin combination
    type PostAuthorOriginValidator: ActorOriginValidator<Self::Origin, Self::PostAuthorId>;

    /// Discussion thread Id type
    type ThreadId: From<u32> + Into<u32> + Parameter + Default + Copy;

    /// Post Id type
    type PostId: From<u32> + Parameter + Default + Copy;

    /// Type for the post author id. Should be authenticated by account id.
    type PostAuthorId: From<Self::AccountId> + Parameter + Default;

    /// Defines post edition number limit.
    type MaxPostEditionNumber: Get<u32>;

    /// Defines thread title length limit.
    type ThreadTitleLengthLimit: Get<u32>;

    /// Defines post length limit.
    type PostLengthLimit: Get<u32>;

    /// Defines max thread by same author in a row number limit.
    type MaxThreadInARowNumber: Get<u32>;
}

// Storage for the proposals discussion module
decl_storage! {
    pub trait Store for Module<T: Trait> as ProposalDiscussion {
        /// Map thread identifier to corresponding thread.
        pub ThreadById get(thread_by_id): map T::ThreadId =>
            Thread<MemberId<T>, T::BlockNumber>;

        /// Count of all threads that have been created.
        pub ThreadCount get(fn thread_count): u32;

        /// Map thread id and post id to corresponding post.
        pub PostThreadIdByPostId: double_map T::ThreadId, twox_128(T::PostId) =>
             Post<T::PostAuthorId, T::BlockNumber, T::ThreadId>;

        /// Count of all posts that have been created.
        pub PostCount get(fn post_count): u32;

        /// Last author thread counter (part of the antispam mechanism)
        pub LastThreadAuthorCounter get(fn last_thread_author_counter):
            Option<ThreadCounter<MemberId<T>>>;
    }
}

decl_module! {
    /// 'Proposal discussion' substrate module
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// Emits an event. Default substrate implementation.
        fn deposit_event() = default;

        /// Adds a post with author origin check.
        pub fn add_post(
            origin,
            post_author_id: T::PostAuthorId,
            thread_id : T::ThreadId,
            text : Vec<u8>
        ) {
            ensure!(
                T::PostAuthorOriginValidator::validate_actor_origin(origin, post_author_id.clone()),
                errors::MSG_INVALID_POST_AUTHOR_ORIGIN
            );
            ensure!(<ThreadById<T>>::exists(thread_id), errors::MSG_THREAD_DOESNT_EXIST);

            ensure!(!text.is_empty(), errors::MSG_EMPTY_POST_PROVIDED);
            ensure!(
                text.len() as u32 <= T::PostLengthLimit::get(),
                errors::MSG_TOO_LONG_POST
            );

            // mutation

            let next_post_count_value = Self::post_count() + 1;
            let new_post_id = next_post_count_value;

            let new_post = Post {
                text,
                created_at: Self::current_block(),
                updated_at: Self::current_block(),
                author_id: post_author_id.clone(),
                edition_number : 0,
                thread_id,
            };

            let post_id = T::PostId::from(new_post_id);
            <PostThreadIdByPostId<T>>::insert(thread_id, post_id, new_post);
            PostCount::put(next_post_count_value);
            Self::deposit_event(RawEvent::PostCreated(post_id, post_author_id));
       }

        /// Updates a post with author origin check. Update attempts number is limited.
        pub fn update_post(
            origin,
            post_author_id: T::PostAuthorId,
            thread_id: T::ThreadId,
            post_id : T::PostId,
            text : Vec<u8>
        ){
            ensure!(
                T::PostAuthorOriginValidator::validate_actor_origin(origin, post_author_id.clone()),
                errors::MSG_INVALID_POST_AUTHOR_ORIGIN
            );

            ensure!(<ThreadById<T>>::exists(thread_id), errors::MSG_THREAD_DOESNT_EXIST);
            ensure!(<PostThreadIdByPostId<T>>::exists(thread_id, post_id), errors::MSG_POST_DOESNT_EXIST);

            ensure!(!text.is_empty(), errors::MSG_EMPTY_POST_PROVIDED);
            ensure!(
                text.len() as u32 <= T::PostLengthLimit::get(),
                errors::MSG_TOO_LONG_POST
            );

            let post = <PostThreadIdByPostId<T>>::get(&thread_id, &post_id);

            ensure!(post.author_id == post_author_id, errors::MSG_NOT_AUTHOR);
            ensure!(post.edition_number < T::MaxPostEditionNumber::get(),
                errors::MSG_POST_EDITION_NUMBER_EXCEEDED);

            let new_post = Post {
                text,
                updated_at: Self::current_block(),
                edition_number: post.edition_number + 1,
                ..post
            };

            // mutation

            <PostThreadIdByPostId<T>>::insert(thread_id, post_id, new_post);
            Self::deposit_event(RawEvent::PostUpdated(post_id, post_author_id));
       }
    }
}

impl<T: Trait> Module<T> {
    // Wrapper-function over system::block_number()
    fn current_block() -> T::BlockNumber {
        <system::Module<T>>::block_number()
    }

    /// Create the discussion thread. Cannot add more threads than 'predefined limit = MaxThreadInARowNumber'
    /// times in a row by the same author.
    pub fn create_thread(
        origin: T::Origin,
        thread_author_id: MemberId<T>,
        title: Vec<u8>,
    ) -> Result<T::ThreadId, &'static str> {
        ensure!(
            T::ThreadAuthorOriginValidator::validate_actor_origin(origin, thread_author_id.clone()),
            errors::MSG_INVALID_THREAD_AUTHOR_ORIGIN
        );

        ensure!(!title.is_empty(), errors::MSG_EMPTY_TITLE_PROVIDED);
        ensure!(
            title.len() as u32 <= T::ThreadTitleLengthLimit::get(),
            errors::MSG_TOO_LONG_TITLE
        );

        // get new 'threads in a row' counter for the author
        let current_thread_counter = Self::get_updated_thread_counter(thread_author_id.clone());

        ensure!(
            current_thread_counter.counter as u32 <= T::MaxThreadInARowNumber::get(),
            errors::MSG_MAX_THREAD_IN_A_ROW_LIMIT_EXCEEDED
        );

        let next_thread_count_value = Self::thread_count() + 1;
        let new_thread_id = next_thread_count_value;

        let new_thread = Thread {
            title,
            created_at: Self::current_block(),
            author_id: thread_author_id.clone(),
        };

        // mutation

        let thread_id = T::ThreadId::from(new_thread_id);
        <ThreadById<T>>::insert(thread_id, new_thread);
        ThreadCount::put(next_thread_count_value);
        <LastThreadAuthorCounter<T>>::put(current_thread_counter);
        Self::deposit_event(RawEvent::ThreadCreated(thread_id, thread_author_id));

        Ok(thread_id)
    }

    // returns incremented thread counter if last thread author equals with provided parameter
    fn get_updated_thread_counter(author_id: MemberId<T>) -> ThreadCounter<MemberId<T>> {
        // if thread counter exists
        if let Some(last_thread_author_counter) = Self::last_thread_author_counter() {
            // if last(previous) author is the same as current author
            if last_thread_author_counter.author_id == author_id {
                return last_thread_author_counter.increment();
            }
        }

        // else return new counter (set with 1 thread number)
        ThreadCounter::new(author_id)
    }
}
