create trigger update_device_updated_at
before update on device
for each row execute procedure update_modified_column();
