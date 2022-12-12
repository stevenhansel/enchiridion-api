-- Add migration script here
create table "device_livestream" (
  time timestamptz not null,
  device_id integer not null references "device"(id),
  num_of_faces integer not null
)
