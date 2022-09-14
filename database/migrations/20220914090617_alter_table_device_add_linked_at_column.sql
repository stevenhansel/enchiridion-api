-- Add migration script here
alter table "device"
add column "linked_at" timestamptz
