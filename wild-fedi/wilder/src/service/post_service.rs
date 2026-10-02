use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::domain::{Account, NewPost, Post};
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
        // Posting into a group requires membership; a personal post does not.
        if let Some(group) = &input.group_username {
            self.groups
                .role_of(group, author.id)
                .await?
                .ok_or_else(|| AppError::Forbidden("not a member of this group".to_string()))?;
        }

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
            published: now,
        };
        self.posts.insert(&post).await?;
        self.activities
            .append_outbox(&author.username, activity_value.clone())
            .await?;

        // Federate to the audience's *remote* followers. Local members read the
        // post from their feed query, so they need no delivery job.
        for follower in self.followers.list(&target).await? {
            if !self.is_local(&follower.actor_id) {
                self.queue
                    .enqueue(Job::Deliver {
                        sender: author.username.clone(),
                        inbox_url: follower.inbox,
                        activity: activity_value.clone(),
                    })
                    .await;
            }
        }

        tracing::info!(author = %author.username, group = ?post.group_username, "published post");
        Ok(post)
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
