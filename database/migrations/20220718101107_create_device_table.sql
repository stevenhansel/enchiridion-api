-- Add migration script here
create table "device" (
  id serial primary key,

  name varchar(255) not null,
  description text not null,

  floor_id integer not null references floor(id),

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
