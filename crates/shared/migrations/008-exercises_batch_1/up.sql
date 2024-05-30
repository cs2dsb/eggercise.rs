-- Note the IDs need to be provided (rather than a generation fn in the db) so they match
-- between the server and client databases

INSERT INTO exercise (id, name) 
VALUES ('3d551aeb-6294-4634-b138-d29159e1ea5d', 'Squat, front (barbell)');
INSERT INTO exercise (id, name) 
VALUES ('5539be81-057a-4b25-92ce-475927d140a2', 'Squat, back (barbell)');
INSERT INTO exercise (id, name) 
VALUES ('553a5fed-905d-4df3-a9c1-53af8ba8bc91', 'Deadlift, rdl (barbell)');
INSERT INTO exercise (id, name) 
VALUES ('5c8c1e48-44ef-437b-9a66-cab3fea26f79', 'Benchpress (barbell)');
INSERT INTO exercise (id, name) 
VALUES ('f7cecea2-ed71-44ef-b301-a47224dce895', 'Benchpress (dumbbell)');
INSERT INTO exercise (id, name) 
VALUES ('e61c4c8e-de48-4653-ac3b-07da2ff2e351', 'Lat pulldown');
INSERT INTO exercise (id, name) 
VALUES ('cbca86c3-b296-4741-a1d1-9b61b6acc191', 'Overhead press (barbell)');
INSERT INTO exercise (id, name) 
VALUES ('16a8d0bc-b08d-4bb1-877d-d98cc096f52c', 'Bicep curl (dumbbell)');
INSERT INTO exercise (id, name) 
VALUES ('fcc517c0-2d3b-421b-b3b2-ce8109996ac1', 'Calf raise');
INSERT INTO exercise (id, name) 
VALUES ('a6be39f2-344a-471f-ad07-8b555f638806', 'Skullcrushers (barbell)');

INSERT INTO exercise_group (id, name)
VALUES ('f448d7a6-a044-4818-9c98-e9f22f2f1fed', 'Primary exercises');
INSERT INTO exercise_group (id, name)
VALUES ('9b358007-2ce7-40c9-8367-497d9c55e50e', 'Accessory exercises');

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Squat, front (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Squat, back (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Deadlift, rdl (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Benchpress (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Benchpress (dumbbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Lat pulldown'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Overhead press (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Bicep curl (dumbbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Calf raise'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (exercise_id, group_id)
VALUES ((SELECT id FROM exercise WHERE name = 'Skullcrushers (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

