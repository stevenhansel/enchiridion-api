-- Add migration script here
create table "device" (
  id serial primary key,

  name varchar(255) unique not null,
  description text not null,

  access_key_id text not null,
  secret_access_key bytea not null,
  secret_access_key_salt text not null,

  floor_id integer not null references floor(id),

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  linked_at timestamptz
);
