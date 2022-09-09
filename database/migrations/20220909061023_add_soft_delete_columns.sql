-- Add migration script here
alter table "building"
add column "deleted_at" timestamptz;

alter table "floor"
add column "deleted_at" timestamptz;

alter table "device"
add column "deleted_at" timestamptz;
