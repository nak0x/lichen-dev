use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::cache::LastVisitStore;
use crate::domain::{Account, Post};
use crate::error::AppError;
use crate::repository::{GroupRepository, PostRepository};

/// How far back a *first* visit reaches, before any last-visit mark exists.
const FIRST_VISIT_WINDOW_DAYS: i64 = 30;
/// Hard cap on a feed batch — this is "since last visit", not infinite scroll.
const FEED_LIMIT: i64 = 200;

/// The personal feed: everything posted in the user's groups since they last
/// looked. Bounded and one-shot by design — the README asks for exactly the new
/// content, not an endless timeline.
pub struct FeedService {
    groups: Arc<dyn GroupRepository>,
    posts: Arc<dyn PostRepository>,
    last_visit: Arc<LastVisitStore>,
}

/// A feed batch plus the moment it covers *from*, so the UI can say "depuis …".
pub struct FeedBatch {
    pub since: DateTime<Utc>,
    pub posts: Vec<Post>,
}

impl FeedService {
    pub fn new(
        groups: Arc<dyn GroupRepository>,
        posts: Arc<dyn PostRepository>,
        last_visit: Arc<LastVisitStore>,
    ) -> Self {
        Self {
            groups,
            posts,
            last_visit,
        }
    }

    /// Build the account's since-last-visit feed, then advance the mark to now so
    /// the next call returns only what is newer. Reading the feed consumes it.
    pub async fn since_last_visit(&self, account: &Account) -> Result<FeedBatch, AppError> {
        let now = Utc::now();
        let since = self
            .last_visit
            .get(&account.username)
            .await?
            .unwrap_or_else(|| now - Duration::days(FIRST_VISIT_WINDOW_DAYS));

        let group_usernames: Vec<String> = self
            .groups
            .for_account(account.id)
            .await?
            .into_iter()
            .map(|g| g.username)
            .collect();

        let posts = if group_usernames.is_empty() {
            Vec::new()
        } else {
            self.posts
                .in_groups_since(&group_usernames, since, FEED_LIMIT)
                .await?
        };

        self.last_visit.set(&account.username, now).await?;
        Ok(FeedBatch { since, posts })
    }
}
