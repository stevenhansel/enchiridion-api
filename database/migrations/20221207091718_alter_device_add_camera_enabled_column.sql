-- Add migration script here
alter table "device" add column "camera_enabled" boolean default false not null
