-- Add migration script here
create table role (
  id serial primary key,
  name varchar(255) not null,
  description text
);
