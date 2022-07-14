-- Add migration script here
create table permission (
  id serial primary key,
  name varchar(255) not null,
  label varchar(255) not null
);
