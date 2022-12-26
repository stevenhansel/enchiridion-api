-- Add migration script here
alter table "announcement" drop column media;
alter table "announcement" drop column media_type;
alter table "announcement" drop column media_duration;
