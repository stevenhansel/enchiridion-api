-- Add migration script here
alter type request_action_type rename to request_action_type_old;

create type request_action_type as enum('create', 'extend_date', 'delete', 'change_devices');

alter table request alter column action type request_action_type using action::text::request_action_type;

drop type request_action_type_old;
