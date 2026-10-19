-- Group trust tiers + post moderation.
--
-- Membership roles grow from admin|member to admin|contributor|guest. The role
-- column is free-form TEXT (validated in the domain), so only the data needs
-- migrating: existing `member` rows could post directly, which is exactly the
-- `contributor` tier, so they map there.
UPDATE memberships SET role = 'contributor' WHERE role = 'member';

-- A guest's submission is held for admin review before it goes live. The status
-- gates both visibility (feeds/group views/pictures read only `published`) and
-- federation (a post is delivered only once published). Defaulting to
-- `published` keeps every existing row and the contributor/admin path unchanged.
ALTER TABLE posts
    ADD COLUMN status TEXT NOT NULL DEFAULT 'published';

-- The review queue: pending posts in a group, newest first.
CREATE INDEX posts_pending_idx
    ON posts (group_username, published DESC)
    WHERE status = 'pending';
