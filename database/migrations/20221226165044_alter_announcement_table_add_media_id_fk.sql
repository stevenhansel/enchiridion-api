-- Add migration script here
alter table "announcement" add column "media_id" integer not null references "media"(id);
