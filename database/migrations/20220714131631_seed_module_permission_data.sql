-- Add migration script here
	insert into module_permission(module_id, permission_id)
	values
  -- announcement
  (1, 1),
  (1, 2),
  (1, 3),

  -- request
  (2, 4),
  (2, 5),
  (2, 6),
  (2, 7),
  
  -- device
  (3, 8),
  (3, 9),
  (3, 10),
  (3, 11),
  
  -- floor
  (4, 12),
  (4, 13),
  (4, 14),
  (4, 15),

  -- building		
  (5, 16),
  (5, 17),
  (5, 18),
  (5, 19),

  -- user
  (6, 20),
  (6, 21);
