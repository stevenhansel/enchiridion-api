-- Add migration script here
create type request_action_type as enum ('create', 'change_date', 'delete', 'change_content', 'change_devices');

create table "request" (
  id serial primary key,

  action request_action_type not null,
  description text not null,
  metadata jsonb,

  approved_by_lsc boolean,
  approved_by_bm boolean,

  announcement_id integer not null references announcement(id),
  user_id integer not null references "user"(id),

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  approval_timestamp timestamptz
)
