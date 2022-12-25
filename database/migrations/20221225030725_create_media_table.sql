-- Add migration script here
create type media_type as enum('image', 'video');

create table "media" (
  "id" serial primary key,
  "path" text not null,
  "media_type" media_type not null,
  "media_duration" float8,
  "created_at" timestamptz not null default now(),
  "updated_at" timestamptz not null default now()
)
