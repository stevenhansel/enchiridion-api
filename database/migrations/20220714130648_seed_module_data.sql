-- Add migration script here
insert into module (name, label)
values
	('announcement', 'Announcement'), -- id = 1
	('request', 'Request'), -- id = 2
	('device', 'Device'), -- id = 3
	('floor', 'Floor'), -- id = 4
	('building', 'Building'), -- id = 5
	('user', 'User'); -- id = 6
