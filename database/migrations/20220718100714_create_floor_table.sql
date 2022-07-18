-- Add migration script here
create table "floor" (
  id serial primary key,

  name varchar(255) unique not null,

  building_id integer not null references building(id),

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
