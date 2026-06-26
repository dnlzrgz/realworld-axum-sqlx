-- Automatically updates the `updated_at` column whenever a row changes.
create or replace function set_updated_at()
  returns trigger as
$$
begin
  NEW.updated_at = now();
  return NEW;
end;
$$ language plpgsql;

-- Creates an `update_at` trigger for the specified table.
create or replace function trigger_updated_at(tablename regclass)
  returns void as
$$
begin
  execute format('CREATE TRIGGER set_updated_at
      BEFORE UPDATE
      ON %s
      FOR EACH ROW
      WHEN (OLD is distinct from NEW)
  EXECUTE FUNCTION set_updated_at();', tablename);
end;
$$ language plpgsql;

-- Useful for columns such as usernames and email addresses where
-- comparisons and UNIQUE constraints.
create collation case_insensitive (
    provider = icu,
    locale = 'und-u-ks-level2',
    deterministic = false
);
