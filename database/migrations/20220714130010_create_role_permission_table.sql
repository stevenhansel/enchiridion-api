-- Add migration script here
create table role_permission (
  role_id integer not null references role (id),
  permission_id integer not null references permission (id),

  primary key (role_id, permission_id)
);
