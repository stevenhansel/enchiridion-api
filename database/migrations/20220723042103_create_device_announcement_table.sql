-- Add migration script here
create table "device_announcement" (
  device_id integer not null references device(id),
  announcement_id integer not null references announcement(id),

  primary key (device_id, announcement_id)
)
