-- Add migration script here
-- LSC, id = 1
insert into role_permission (role_id, permission_id)
values
-- announcement
(1, 1),
(1, 2),
(1, 3),

-- request
(1, 4),
(1, 5),
(1, 6),
(1, 7),

-- device
(1, 8),
(1, 9),

-- floor
(1, 12),

-- building
(1, 16),

-- user
(1, 20);

-- BM, id = 2
insert into role_permission (role_id, permission_id)
values
-- announcement
(2, 1),
(2, 2),
(2, 3),

-- request
(2, 4),
(2, 5),
(2, 6),
(2, 7),

-- device
(2, 8),
(2, 9),

-- floor
(2, 12),

-- building
(2, 16),

-- user
(2, 20);

-- Student, id = 3
insert into role_permission (role_id, permission_id)
values
-- announcement
(3, 1),
(3, 2),
(3, 3),

-- request
(3, 4),
(3, 5),
(3, 6),

-- device
(3, 8),
(3, 9),

-- floor
(3, 12),

-- building
(3, 16),

-- user
(3, 20);

-- Admin, id = 4
insert into role_permission (role_id, permission_id)
values
-- announcement
(4, 1),
(4, 2),
(4, 3),

-- request
(4, 4),
(4, 5),
(4, 6),
(4, 7),

-- device
(4, 8),
(4, 9),
(4, 10),
(4, 11),

-- floor
(4, 12),
(4, 13),
(4, 14),
(4, 15),

-- building		
(4, 16),
(4, 17),
(4, 18),
(4, 19),

-- user
(4, 20),
(4, 21)
