-- Tracks follow-relationships.
create table follow (
  -- If a user is deleted, their follow row is deleted too.
  follower_user_id uuid not null references users (user_id) on delete cascade,
  following_user_id uuid not null references users (user_id) on delete cascade,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),

  constraint user_cannot_follow_self check (follower_user_id != following_user_id),

  primary key (follower_user_id, following_user_id)
);

-- Without this, deleting an user will cause a sequential scan.
create index idx_follow_following_user_id on follow (following_user_id);

select trigger_updated_at('follow');
