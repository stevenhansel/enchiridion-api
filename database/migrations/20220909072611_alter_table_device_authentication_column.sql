-- Add migration script here
alter table "device"
add column "access_key_id" text not null;

alter table "device"
add column "secret_access_key" bytea(60) not null;
