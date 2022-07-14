-- Add migration script here
create table module_permission (
  module_id integer not null references module (id),
  permission_id integer not null references permission (id),

  primary key (module_id, permission_id)
);
