-- Note the IDs need to be provided (rather than a generation fn in the db) so they match
-- between the server and client databases

INSERT INTO user (id, username, push_notification_subscription) 
VALUES ('cb4d23ae-ac0e-455e-ab70-c16c65894009', 'daniel', '{
  "endpoint": "https://updates.push.services.mozilla.com/wpush/v2/gAAAAABmfZ1to_S5Rch9W0YTKrweajQpOdtBK18jNLEHN7MaHJXBOzrQ6N7-c77Au8_ifhcaL1NTYmVx5dAVBbWSqs2fdlioc9Gedg_4yLJxsI57Y5gMoUnzd9B3AsKddTtRJ5SQ93IAfFOgErpKIK9x_b7Tb4JrkO2xdA1acM9sZL-u3gx5dvw",
  "key": "BJfDBLhI5TdKNWqChltn36zYmHHYrforWD94jJ3A98cXmclUrOId5HZDnQuH1WEn4zR6pSR2l0Tnat5fZL9yEV0=",
  "auth": "haauF_uaL24NyIk_yZYaVQ=="
}');

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

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('bcc6e371-d866-42cf-9962-d3c2e075d920',
        (SELECT id FROM exercise WHERE name = 'Squat, front (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('ecdee95e-4998-48bc-9eb8-fc5705517c84',
        (SELECT id FROM exercise WHERE name = 'Squat, back (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('26851e25-f98e-40a5-aa8d-9fd349717780',
        (SELECT id FROM exercise WHERE name = 'Deadlift, rdl (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('33e80e20-7d68-4ef9-9909-a95cad063983',
        (SELECT id FROM exercise WHERE name = 'Benchpress (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('a549d1be-a2fd-4cb4-8aa9-32efdf14ebd1',
        (SELECT id FROM exercise WHERE name = 'Benchpress (dumbbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('640eda9c-db75-4934-8c17-9bb8d56f7d9b',
        (SELECT id FROM exercise WHERE name = 'Lat pulldown'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('4e72646c-824d-4f13-92de-4507573308ca',
        (SELECT id FROM exercise WHERE name = 'Overhead press (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('d425c5d1-3a54-4d81-a468-d32a217d0bad',
        (SELECT id FROM exercise WHERE name = 'Bicep curl (dumbbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('429ec3b4-d974-4d90-95b7-89103f463731',
        (SELECT id FROM exercise WHERE name = 'Calf raise'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO exercise_group_member (id, exercise_id, group_id)
VALUES ('b499094a-273f-4d5f-8e88-193bb7f80318',
        (SELECT id FROM exercise WHERE name = 'Skullcrushers (barbell)'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO plan (id, owner_id, name, duration_weeks) 
VALUES ('173cbd20-4fa0-4de4-bed8-40eb2111a92d',
        (SELECT id FROM user WHERE username = 'daniel'),
        'Weekly Undulating',
        15);

INSERT INTO plan_exercise_group (id, plan_id, exercise_group_id)
VALUES ('5f7d726a-a440-4afa-9e20-b5108442a34d',
        (SELECT id FROM plan WHERE name = 'Weekly Undulating'),
        (SELECT id FROM exercise_group WHERE name = 'Primary exercises'));

INSERT INTO plan_exercise_group (id, plan_id, exercise_group_id)
VALUES ('ef1a6ad2-1005-4ed1-ace6-5dbf4634b1d3',
        (SELECT id FROM plan WHERE name = 'Weekly Undulating'),
        (SELECT id FROM exercise_group WHERE name = 'Accessory exercises'));

INSERT INTO plan_instance (id, plan_id, user_id, start_date)
VALUES ('d90f7e50-77f5-49cf-b70a-f98fdfc3a410',
        (SELECT id FROM plan WHERE name = 'Weekly Undulating'),
        (SELECT id FROM user WHERE username = 'daniel'),
        CURRENT_TIMESTAMP);
