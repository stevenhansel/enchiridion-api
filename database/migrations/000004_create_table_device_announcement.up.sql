create table "device_announcement" (
  announcement_id int, 
  device_id int,

  constraint device_announcement_key unique (announcement_id, device_id),
  foreign key (announcement_id) references announcement (id),
  foreign key (device_id) references device (id)
)
