create table articles (
  article_id uuid primary key default uuidv7(),
  user_id uuid not null references users (user_id) on delete cascade,

  slug text not null unique,
  title text not null,
  description text not null,
  body text not null,

  -- Postgres let's us store the tags as an array.
  tag_list text[] not null,

  -- Both are required by the RealWorld spec.
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

select trigger_updated_at('articles');

-- Speed up searching with tags.
create index article_tags_gin on articles using gin (tag_list);

create table favorites (
  article_id uuid not null references articles (article_id) on delete cascade,
  user_id uuid not null references users (user_id) on delete cascade,

  primary key (article_id, user_id)
);

select trigger_updated_at('favorites');

create table comments (
  comment_id uuid primary key default uuidv7(),
  article_id uuid not null references articles (article_id) on delete cascade,
  user_id uuid not null references users (user_id) on delete cascade,

  body text not null,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

select trigger_updated_at('comments');

create index on comments (article_id, created_at);
