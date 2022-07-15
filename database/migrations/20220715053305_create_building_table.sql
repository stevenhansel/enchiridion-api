-- Add migration script here
create table "building" (
  id serial primary key,

  name varchar(255) unique not null,
  color varchar(7) not null,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
