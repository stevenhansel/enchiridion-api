-- Add migration script here
create type user_status as enum ('waiting_for_approval', 'approved', 'rejected');

create table if not exists  "user" (
  id serial primary key,
  name varchar(255) not null,
  password bytea not null,
  registration_reason text not null,
  is_email_confirmed boolean default false,
  status user_status default 'waiting_for_approval',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);
