-- Add migration script here
insert into permission (name, label)
values
	-- announcement, module id = 1
	('view_list_announcement', 'View List Announcement'), -- id = 1
	('view_announcement_detail', 'View Announcement Detail'), -- id = 2
	('create_announcement', 'Create Announcement'), -- id = 3

	-- request, module id = 2
	('view_list_request', 'View List Request'), -- id = 4
	('create_request', 'Create Request'), -- id = 5
	('cancel_request', 'Cancel Request'), -- id = 6
	('approve_reject_request', 'Approve/Reject Request'), -- id = 7
	
	-- device, module id = 3
	('view_list_device', 'View List Device'), -- id = 8
	('view_device_detail', 'View Device Detail'), -- id = 9
	('update_device', 'Update Device'), -- id = 10
	('unlink_device', 'Unlink Device'), -- id = 11

	-- floor, module id = 4
	('view_list_floor', 'View List Floor'), -- id = 12
	('create_floor', 'Create Floor'), -- id = 13
	('update_floor', 'Update Floor'), -- id = 14
	('delete_floor', 'Delete Floor'), -- id = 15

	-- building, module id = 5
	('view_list_building', 'View List Building'), -- id = 16
	('create_building', 'Create Building'), -- id = 17
	('update_building', 'Update Building'), -- id = 18
	('delete_building', 'Delete Building'), -- id = 19

	-- user, module id = 6
	('view_list_user', 'View List User'), -- id = 20
	('approve_reject_user', 'Approve/Reject User'); -- id = 21
