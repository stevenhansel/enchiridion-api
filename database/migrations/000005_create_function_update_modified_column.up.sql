create function update_modified_column()   
returns trigger as $$
begin
    NEW.updated_at = now();
    return NEW;   
end;
$$ language 'plpgsql';
