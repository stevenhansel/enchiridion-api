-- Add migration script here
create type announcement_status as enum ('waiting_for_approval', 'active', 'done', 'canceled', 'rejected');


create table "announcement" (
  id serial primary key,
  title varchar(255) not null,
  media text not null,
  start_date timestamptz not null,
  end_date timestamptz not null,
  status announcement_status not null default 'waiting_for_approval',
  notes text not null,
  user_id  integer not null references "user"(id),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  approved_at timestamptz
)
