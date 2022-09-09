-- Add migration script here
alter table "user"
add column "password_salt" text not null
