-- Add migration script here
alter table "request"
add column lsc_approver integer references "user"(id),
add column bm_approver integer references "user"(id)
