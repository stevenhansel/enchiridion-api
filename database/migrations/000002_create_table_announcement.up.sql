create table "announcement" (
  id serial primary key,
  name varchar(255) not null,
  media text not null,
  status announcement_status default 'waiting_for_approval' not null,
  notes text not null,
  rejection_notes text,
  approved_at timestamptz,
  created_at timestamptz default current_timestamp not null,
  updated_at timestamptz default current_timestamp not null
)
