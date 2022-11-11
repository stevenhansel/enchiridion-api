-- Add migration script here
alter table "user" add column "building_id" int references building(id);
