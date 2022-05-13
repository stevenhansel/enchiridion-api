create type announcement_status as enum (
  'waiting_for_approval',
  'waiting_for_sync',
  'active',
  'done',
  'canceled',
  'rejected'
)
