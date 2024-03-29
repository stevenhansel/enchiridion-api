-- Add migration script here
create type user_status as enum ('waiting_for_approval', 'approved', 'rejected');

create table "user" (
  id serial primary key,

  name varchar(255) not null,
  email varchar(255) unique not null,
  password bytea not null,
  registration_reason text,

  profile_picture text,

  is_email_confirmed boolean not null default false,
  status user_status not null default 'waiting_for_approval',

  role varchar(255) not null,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
