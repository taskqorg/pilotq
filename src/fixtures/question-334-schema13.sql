-- Invented pre-upgrade schema-13 state: no retirement event existed.
DELETE FROM outbox;
UPDATE requests SET state='answered' WHERE id=1;
INSERT INTO projects(name,repo,root,by,at) VALUES ('pilotq','invented/pilotq','/invented/pilotq',7,0);
INSERT INTO tasks(number,request_id,chat,topic,state,revision,observed,closed,project,board,root)
VALUES (19,1,7,NULL,'review',30,3,1,'pilotq','invented/pilotq','/invented/pilotq');
INSERT INTO outbox(request_id,chat,topic,text,state,next_at,sent_message,kind,project,task,revision,question) VALUES
(1,7,NULL,'Invented old question','delivered',1,110,'question','pilotq',19,10,'{"parsed":true,"choices":[{"id":1,"label":"Invented A"}]}'),
(1,7,NULL,'Invented current question','delivered',2,120,'question','pilotq',19,20,'{"parsed":true,"choices":[{"id":1,"label":"Invented B"}]}'),
(1,7,NULL,'Answered by Invented Actor: option 1','delivered',3,120,'question_answered','pilotq',19,20,NULL),
(1,7,NULL,'Invented image receipt','delivered',2,121,'result_image','pilotq',19,20,NULL);
