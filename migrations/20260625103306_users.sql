-- User accounts.
create table users (
  -- UUIDv7 provides unique, time-ordered identifiers with better
  -- index locality.
  user_id uuid primary key default uuidv7(),

  -- Usernames should be unique.
  username text collate "case_insensitive" unique not null,
  
  -- Email addresses should be unique.
  email text collate "case_insensitive" unique not null,

  -- User profile biography.
  bio text not null default '',

  -- Optional external profile image.
  image text,

  -- Argon2 password hash.
  password_hash text not null,

  -- Timestamp when the account was created.
  created_at timestamptz not null default now(),

  -- Timestamp of the most recent modification.
  -- Automatically maintained by the `set_updated_at()` trigger.
  updated_at timestamptz not null default now()
);

-- Automatically update `updated_at` whenever a row changes.
select trigger_updated_at('users');
