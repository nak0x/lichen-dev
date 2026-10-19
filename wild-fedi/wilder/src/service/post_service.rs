use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{Account, NewPost, Post, PostStatus};
use crate::error::AppError;
use crate::queue::{Job, JobQueue};
use crate::repository::{
    ActivityRepository, FollowerRepository, GroupRepository, MediaRepository, PostRepository,
};
use crate::vocab::{self, Activity, Object};

/// Composing and publishing posts. One `create` call does three things that must
/// stay in step: persist the read-model row, append the `Create` to the author's
/// outbox for federation, and fan it out to the audience's remote followers.
pub struct PostService {
    config: Arc<AppConfig>,
    groups: Arc<dyn GroupRepository>,
    followers: Arc<dyn FollowerRepository>,
    posts: Arc<dyn PostRepository>,
    activities: Arc<dyn ActivityRepository>,
    media: Arc<dyn MediaRepository>,
    queue: Arc<dyn JobQueue>,
}

impl PostService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<AppConfig>,
        groups: Arc<dyn GroupRepository>,
        followers: Arc<dyn FollowerRepository>,
        posts: Arc<dyn PostRepository>,
        activities: Arc<dyn ActivityRepository>,
        media: Arc<dyn MediaRepository>,
        queue: Arc<dyn JobQueue>,
    ) -> Self {
        Self {
            config,
            groups,
            followers,
            posts,
            activities,
            media,
            queue,
        }
    }

    pub async fn create(&self, author: &Account, input: NewPost) -> Result<Post, AppError> {
        // Bound the content so one post can't carry an unreasonable payload.
        if input.content.trim().is_empty() {
            return Err(AppError::BadRequest("content is required".to_string()));
        }
        if input.content.len() > 5_000 {
            return Err(AppError::BadRequest("content too long (max 5000)".to_string()));
        }
        // Posting into a group requires membership; a personal post does not. The
        // member's tier also decides the outcome: a guest's group submission is
        // held for admin review (Pending); contributors and admins publish
        // directly. Personal posts have no group admin, so they publish directly.
        let status = if let Some(group) = &input.group_username {
            let role = self
                .groups
                .role_of(group, author.id)
                .await?
                .ok_or_else(|| AppError::Forbidden("not a member of this group".to_string()))?;
            if role.can_post_without_review() {
                PostStatus::Published
            } else {
                PostStatus::Pending
            }
        } else {
            PostStatus::Published
        };

        // The audience's actor: the group amplifies to its followers; a personal
        // post reaches the author's own followers.
        let target = input
            .group_username
            .clone()
            .unwrap_or_else(|| author.username.clone());

        let now = Utc::now();
        let published = now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        let author_url = self.config.actor_url(&author.username);
        let audience_followers = self.config.followers_url(&target);
        let object_id = self.config.object_url(&Uuid::new_v4().to_string());

        // Resolve attached media to AS `Image` objects. Unknown/non-image ids are
        // skipped rather than failing the whole post.
        let mut attachment = Vec::new();
        for media_id in &input.media {
            if let Some(m) = self.media.by_id(*media_id).await?
                && m.content_type.starts_with("image/")
            {
                attachment.push(serde_json::json!({
                    "type": "Image",
                    "mediaType": m.content_type,
                    "url": self.config.media_url(&m.id.to_string()),
                }));
            }
        }

        let note = Object {
            kind: "Note".to_string(),
            id: object_id.clone(),
            attributed_to: Some(author_url.clone()),
            content: Some(input.content.clone()),
            published: Some(published.clone()),
            to: vec![vocab::PUBLIC.to_string()],
            cc: vec![audience_followers.clone()],
            attachment,
            tag: input
                .tags
                .iter()
                .map(|t| serde_json::json!({ "type": "Hashtag", "name": hashtagify(t) }))
                .collect(),
            location: None,
        };
        let activity = Activity {
            context: vocab::activity_context(),
            kind: "Create".to_string(),
            id: self.config.activity_url(&Uuid::new_v4().to_string()),
            actor: author_url,
            to: vec![vocab::PUBLIC.to_string()],
            cc: vec![audience_followers],
            published: Some(published),
            object: serde_json::to_value(&note)?,
        };
        let activity_value = serde_json::to_value(&activity)?;

        let post = Post {
            id: Uuid::new_v4(),
            author_username: author.username.clone(),
            group_username: input.group_username.clone(),
            content: Some(input.content),
            tags: input.tags,
            object_id,
            activity: activity_value.clone(),
            status,
            published: now,
        };
        self.posts.insert(&post).await?;

        if status == PostStatus::Pending {
            // Held for review: no outbox entry, no federation. An admin's approval
            // runs the publish step (`deliver`) later, replaying this activity.
            tracing::info!(author = %author.username, group = ?post.group_username, "submission held for review");
        } else {
            self.deliver(&author.username, &target, &activity_value)
                .await?;
            tracing::info!(author = %author.username, group = ?post.group_username, "published post");
        }
        Ok(post)
    }

    /// Append the activity to the author's outbox and fan it out to the
    /// audience's *remote* followers. Local members read the post from their feed
    /// query, so they need no delivery job. Shared by direct publishing and by an
    /// admin approving a held submission.
    async fn deliver(
        &self,
        author_username: &str,
        target: &str,
        activity: &serde_json::Value,
    ) -> Result<(), AppError> {
        self.activities
            .append_outbox(author_username, activity.clone())
            .await?;
        for follower in self.followers.list(target).await? {
            if !self.is_local(&follower.actor_id) {
                self.queue
                    .enqueue(Job::Deliver {
                        sender: author_username.to_string(),
                        inbox_url: follower.inbox,
                        activity: activity.clone(),
                    })
                    .await;
            }
        }
        Ok(())
    }

    /// The admin review queue for a group: posts a guest submitted that are
    /// awaiting a decision. Caller must be an admin of the group.
    pub async fn pending(
        &self,
        moderator: &Account,
        group_username: &str,
    ) -> Result<Vec<Post>, AppError> {
        self.ensure_moderator(moderator, group_username).await?;
        Ok(self.posts.pending_in_group(group_username, 200).await?)
    }

    /// Approve a held submission: flip it to published and run the deferred
    /// publish (outbox + federation). Caller must be an admin of the post's group.
    pub async fn approve(&self, moderator: &Account, post_id: Uuid) -> Result<Post, AppError> {
        let mut post = self.load_pending(moderator, post_id).await?;
        self.posts.set_status(post.id, PostStatus::Published).await?;
        post.status = PostStatus::Published;

        let target = post
            .group_username
            .clone()
            .unwrap_or_else(|| post.author_username.clone());
        self.deliver(&post.author_username, &target, &post.activity)
            .await?;
        tracing::info!(post = %post.id, by = %moderator.username, "approved submission");
        Ok(post)
    }

    /// Decline a held submission: it stays recorded (so the author can be told)
    /// but is never surfaced nor federated. Caller must be an admin of the group.
    pub async fn reject(&self, moderator: &Account, post_id: Uuid) -> Result<(), AppError> {
        let post = self.load_pending(moderator, post_id).await?;
        self.posts.set_status(post.id, PostStatus::Rejected).await?;
        tracing::info!(post = %post.id, by = %moderator.username, "rejected submission");
        Ok(())
    }

    /// Load a post that must be `Pending` and in a group the caller administers —
    /// the shared guard behind approve/reject.
    async fn load_pending(&self, moderator: &Account, post_id: Uuid) -> Result<Post, AppError> {
        let post = self.posts.by_id(post_id).await?.ok_or(AppError::NotFound)?;
        let group = post
            .group_username
            .clone()
            .ok_or_else(|| AppError::BadRequest("post has no group to moderate".to_string()))?;
        self.ensure_moderator(moderator, &group).await?;
        if post.status != PostStatus::Pending {
            return Err(AppError::Conflict("post is not awaiting review".to_string()));
        }
        Ok(post)
    }

    /// Guard: the account must be an admin of the group.
    async fn ensure_moderator(
        &self,
        account: &Account,
        group_username: &str,
    ) -> Result<(), AppError> {
        let role = self
            .groups
            .role_of(group_username, account.id)
            .await?
            .ok_or_else(|| AppError::Forbidden("not a member of this group".to_string()))?;
        if !role.can_moderate() {
            return Err(AppError::Forbidden("admin only".to_string()));
        }
        Ok(())
    }

    pub async fn in_group(
        &self,
        group_username: &str,
        tag: Option<&str>,
    ) -> Result<Vec<Post>, AppError> {
        Ok(self.posts.in_group(group_username, tag, 100).await?)
    }

    pub async fn captured(&self, group_username: Option<&str>) -> Result<Vec<Post>, AppError> {
        Ok(self.posts.captured(group_username, 100).await?)
    }

    /// The group's picture space: every image attached to a group post, flattened
    /// into `{ url, author, post_id }` entries, newest first.
    pub async fn pictures(&self, group_username: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let posts = self.posts.in_group(group_username, None, 200).await?;
        let mut pics = Vec::new();
        for post in posts {
            let attachments = post
                .activity
                .get("object")
                .and_then(|o| o.get("attachment"))
                .and_then(|a| a.as_array())
                .cloned()
                .unwrap_or_default();
            for att in attachments {
                if att.get("type").and_then(|t| t.as_str()) == Some("Image")
                    && let Some(url) = att.get("url").and_then(|u| u.as_str())
                {
                    pics.push(serde_json::json!({
                        "url": url,
                        "author": post.author_username,
                        "post_id": post.id,
                    }));
                }
            }
        }
        Ok(pics)
    }

    fn is_local(&self, actor_id: &str) -> bool {
        actor_id.starts_with(&self.config.base_url())
    }
}

/// Normalise a user-supplied tag to a `#hashtag` (one leading '#', no spaces).
fn hashtagify(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('#').replace(char::is_whitespace, "-");
    format!("#{trimmed}")
}
