-- Add migration script here
update "request"
  set "metadata" = '{}'::jsonb
where "metadata" is null;

alter table "request" alter column "metadata" set default '{}'::jsonb;
alter table "request" alter column "metadata" set not null;
