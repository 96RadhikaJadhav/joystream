#![warn(missing_docs)]

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::vec::Vec;

/// Represents a discussion thread
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct DiscussionThread<ThreadAuthorId, BlockNumber, MemberId> {
    /// When thread was established.
    pub activated_at: BlockNumber,

    /// Author of the thread.
    pub author_id: ThreadAuthorId,

    /// Thread permission mode.
    pub mode: ThreadMode<MemberId>,
}

/// Post for the discussion thread
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct DiscussionPost<PostAuthorId> {
    /// Author of the post.
    pub author_id: PostAuthorId,
}

/// Post for the discussion thread
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, Copy, PartialEq, Eq)]
pub struct ThreadCounter<ThreadAuthorId> {
    /// Author of the threads.
    pub author_id: ThreadAuthorId,

    /// ThreadCount
    pub counter: u32,
}

impl<ThreadAuthorId: Clone> ThreadCounter<ThreadAuthorId> {
    /// Increments existing counter
    pub fn increment(&self) -> Self {
        ThreadCounter {
            counter: self.counter + 1,
            author_id: self.author_id.clone(),
        }
    }

    /// Creates new counter by author_id. Counter instantiated with 1.
    pub fn new(author_id: ThreadAuthorId) -> Self {
        ThreadCounter {
            author_id,
            counter: 1,
        }
    }
}

/// Discussion thread permission modes.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum ThreadMode<MemberId> {
    /// Every member can post on the thread.
    Open,

    /// Only author, councilor or white member list could post on the thread.
    Closed(Vec<MemberId>),
}

impl<MemberId> Default for ThreadMode<MemberId> {
    fn default() -> Self {
        Self::Open
    }
}
