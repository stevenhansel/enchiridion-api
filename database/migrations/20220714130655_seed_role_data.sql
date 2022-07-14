-- Add migration script here
insert into role (name, description)
values
	('LSC', 'Lecturer Service Center'), -- id = 1
	('BM', 'Building Management'), -- id = 2
	('Student', 'Student enrolled in Bina Nusantara University'), -- id = 3
	('Admin', 'Superadmin of the application'); -- id = 4
