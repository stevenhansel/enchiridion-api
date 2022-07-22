-- Add migration script here
create or replace function update_modified_column()   
returns trigger as $$
begin
    new.updated_at = now();
    return new;   
end;
$$ language 'plpgsql';
