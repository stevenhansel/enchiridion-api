-- Add migration script here
alter table "announcement"
add column "media_type" text,
add column "media_duration" float