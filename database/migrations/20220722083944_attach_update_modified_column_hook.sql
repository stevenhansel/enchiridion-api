-- Add migration script here
create trigger update_user_updated_at_column before update on "user" for each row execute procedure update_modified_column();

create trigger update_building_updated_at_column before update on "building" for each row execute procedure update_modified_column();

create trigger update_floor_updated_at_column before update on "floor" for each row execute procedure update_modified_column();

create trigger update_device_updated_at_column before update on "device" for each row execute procedure update_modified_column();

create trigger update_announcement_updated_at_column before update on "announcement" for each row execute procedure update_modified_column();

create trigger update_request_updated_at_column before update on "request" for each row execute procedure update_modified_column();
