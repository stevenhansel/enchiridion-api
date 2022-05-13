create table "device" (
  id serial primary key,
  name varchar(255),
  description text,
  machine_id text not null,
  created_at timestamptz default current_timestamp not null,
  updated_at timestamptz default current_timestamp not null
)
