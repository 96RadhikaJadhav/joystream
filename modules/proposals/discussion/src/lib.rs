//! Proposals discussion module for the Joystream platform. Version 2.
//! Contains discussion subsystem for the proposals engine.
//!
//! Supported extrinsics:
//!
//!

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

// Do not delete! Cannot be uncommented by default, because of Parity decl_module! issue.
//#![warn(missing_docs)]

#[cfg(test)]
mod tests;
mod types;

use rstd::clone::Clone;
use rstd::prelude::*;
use rstd::vec::Vec;
use runtime_primitives::traits::EnsureOrigin;
use srml_support::{decl_module, decl_storage, Parameter};

use types::{Post, Thread};

//TODO: create_thread() ensures
//TODO: create_post() ensures
//TODO: select storage container for the posts (double map, inside thread)?
//TODO: events?

/// 'Proposal discussion' substrate module Trait
pub trait Trait: system::Trait {
    /// Origin from which thread author must come.
    type ThreadAuthorOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

    /// Origin from which commenter must come.
    type PostAuthorOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;

    /// Discussion thread Id type
    type ThreadId: From<u32> + Into<u32> + Parameter + Default + Copy;

    /// Post Id type
    type PostId: From<u32> + Parameter + Default + Copy;

    /// Type for the thread author id. Should be authenticated by account id.
    type ThreadAuthorId: From<Self::AccountId> + Parameter + Default;

    /// Type for the post author id. Should be authenticated by account id.
    type PostAuthorId: From<Self::AccountId> + Parameter + Default;
}

// Storage for the proposals discussion module
decl_storage! {
    pub trait Store for Module<T: Trait> as ProposalDiscussion {
        /// Map thread identifier to corresponding thread.
        pub ThreadById get(thread_by_id): map T::ThreadId =>
            Thread<T::ThreadAuthorId, T::BlockNumber>;

        /// Count of all threads that have been created.
        pub ThreadCount get(fn thread_count): u32;

        /// Map post id to corresponding post.
        pub PostById get(post_by_id): map T::PostId =>
             Post<T::PostAuthorId, T::BlockNumber, T::ThreadId>;

        /// Count of all posts that have been created.
        pub PostCount get(fn post_count): u32;
    }
}

decl_module! {
    /// 'Proposal discussion' substrate module
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
       pub fn add_post(origin, thread_id : T::ThreadId, text : Vec<u8>) {
            let account_id = T::ThreadAuthorOrigin::ensure_origin(origin)?;
            let post_author_id = T::PostAuthorId::from(account_id);

            let next_post_count_value = Self::post_count() + 1;
            let new_post_id = next_post_count_value;

            let new_post = Post {
                text,
                created_at: Self::current_block(),
                author_id: post_author_id,
                thread_id,
            };

            let post_id = T::PostId::from(new_post_id);
            <PostById<T>>::insert(post_id, new_post);
            PostCount::put(next_post_count_value);
       }
    }
}

impl<T: Trait> Module<T> {
    // Wrapper-function over system::block_number()
    fn current_block() -> T::BlockNumber {
        <system::Module<T>>::block_number()
    }

    /// Create the discussion
    pub fn create_discussion(
        origin: T::Origin,
        title: Vec<u8>,
    ) -> Result<T::ThreadId, &'static str> {
        let account_id = T::ThreadAuthorOrigin::ensure_origin(origin)?;
        let thread_author_id = T::ThreadAuthorId::from(account_id);

        let next_thread_count_value = Self::thread_count() + 1;
        let new_thread_id = next_thread_count_value;

        let new_thread = Thread {
            title,
            created_at: Self::current_block(),
            author_id: thread_author_id,
        };

        let thread_id = T::ThreadId::from(new_thread_id);
        <ThreadById<T>>::insert(thread_id, new_thread);
        ThreadCount::put(next_thread_count_value);

        Ok(thread_id)
    }
}
