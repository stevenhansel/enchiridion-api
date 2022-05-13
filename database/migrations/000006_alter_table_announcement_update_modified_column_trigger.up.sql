create trigger update_announcement_updated_at
before update on announcement
for each row execute procedure update_modified_column();
