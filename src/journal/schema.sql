-- Fresh schema 14 (pilotq.md §9); schema 13 upgrades in schema.rs.
CREATE TABLE admin_ops(request INTEGER NOT NULL, tool TEXT NOT NULL, args TEXT NOT NULL,
                    external TEXT, receipt TEXT, PRIMARY KEY(request,tool));
CREATE TABLE audit_events(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, run_id INTEGER, at INTEGER NOT NULL, kind TEXT NOT NULL,
                    target TEXT, item TEXT, result TEXT);
CREATE TABLE audit_reads(id INTEGER PRIMARY KEY, at INTEGER NOT NULL, reader INTEGER NOT NULL, front TEXT NOT NULL, filter TEXT NOT NULL,
                    count INTEGER NOT NULL);
CREATE TABLE cards(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL UNIQUE, generation INTEGER NOT NULL,
                    plan_id INTEGER, question TEXT, options TEXT, used_request INTEGER);
CREATE TABLE conversations(session TEXT PRIMARY KEY, generation INTEGER NOT NULL DEFAULT 0, thread TEXT, fence INTEGER NOT NULL DEFAULT 0, project TEXT, project_by INTEGER, project_at INTEGER, runtime TEXT);
CREATE TABLE effects(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, op_id TEXT NOT NULL UNIQUE, tool TEXT NOT NULL, repo TEXT NOT NULL,
                    issue INTEGER NOT NULL, value TEXT NOT NULL, key TEXT NOT NULL, state TEXT NOT NULL, receipt TEXT, at INTEGER NOT NULL);
CREATE TABLE facts(subject INTEGER NOT NULL, key TEXT NOT NULL, value TEXT NOT NULL, private INTEGER NOT NULL, source TEXT NOT NULL,
                revision INTEGER NOT NULL, at INTEGER NOT NULL, PRIMARY KEY(subject, key));
CREATE TABLE grant_log(id INTEGER PRIMARY KEY, at INTEGER NOT NULL, actor INTEGER, target INTEGER NOT NULL, before TEXT NOT NULL, after TEXT NOT NULL);
CREATE TABLE grants(user INTEGER NOT NULL, name TEXT NOT NULL, PRIMARY KEY(user, name));
CREATE TABLE health(source TEXT PRIMARY KEY, at INTEGER NOT NULL, state TEXT NOT NULL, reason TEXT);
CREATE TABLE meta(key TEXT PRIMARY KEY, value INTEGER NOT NULL);
CREATE TABLE outbox(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, chat INTEGER NOT NULL, topic INTEGER, reply_to INTEGER,
                html TEXT, text TEXT NOT NULL, state TEXT NOT NULL DEFAULT 'pending', attempts INTEGER NOT NULL DEFAULT 0, next_at INTEGER NOT NULL,
                possible_duplicate INTEGER NOT NULL DEFAULT 0, sent_message INTEGER, error TEXT, kind TEXT NOT NULL DEFAULT 'reply', task INTEGER, revision INTEGER, project TEXT NOT NULL DEFAULT 'pilotq', question TEXT);
CREATE TABLE pilot_notes(id INTEGER PRIMARY KEY, at INTEGER NOT NULL, by INTEGER NOT NULL, kind TEXT NOT NULL, text TEXT NOT NULL);
CREATE TABLE plan_runs(id INTEGER PRIMARY KEY, plan_id INTEGER NOT NULL UNIQUE, request_id INTEGER NOT NULL, actor INTEGER NOT NULL,
                    state TEXT NOT NULL, reason TEXT, next_at INTEGER NOT NULL, at INTEGER NOT NULL);
CREATE TABLE plans(id INTEGER PRIMARY KEY, session TEXT NOT NULL, revision INTEGER NOT NULL, request_id INTEGER NOT NULL, tickets TEXT NOT NULL,
                    at INTEGER NOT NULL, project TEXT NOT NULL DEFAULT 'pilotq', board TEXT NOT NULL DEFAULT 'taskqorg/pilotq', root TEXT NOT NULL DEFAULT '', draft INTEGER, UNIQUE(session, revision));
CREATE TABLE projects(name TEXT PRIMARY KEY, repo TEXT NOT NULL, root TEXT NOT NULL, by INTEGER NOT NULL, at INTEGER NOT NULL);
CREATE TABLE requests(id INTEGER PRIMARY KEY, update_id INTEGER NOT NULL UNIQUE, chat INTEGER NOT NULL, topic INTEGER,
                sender INTEGER NOT NULL, message INTEGER NOT NULL, kind TEXT NOT NULL, text TEXT, state TEXT NOT NULL, received_at INTEGER NOT NULL, session TEXT, front TEXT NOT NULL DEFAULT 'telegram', reply TEXT, error TEXT, name TEXT, grants TEXT, project TEXT, reply_to INTEGER, image TEXT, language TEXT);
CREATE TABLE route_projects(chat INTEGER NOT NULL, topic INTEGER NOT NULL, project TEXT NOT NULL,
                    PRIMARY KEY(chat,topic));
CREATE TABLE routes(chat INTEGER NOT NULL, topic INTEGER NOT NULL, shared INTEGER NOT NULL, by INTEGER NOT NULL, at INTEGER NOT NULL,
                PRIMARY KEY(chat, topic));
CREATE TABLE runs(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, session TEXT NOT NULL, generation INTEGER NOT NULL, fence INTEGER NOT NULL,
                    state TEXT NOT NULL, tool INTEGER NOT NULL DEFAULT 0, started_at INTEGER NOT NULL, ended_at INTEGER, outcome TEXT, model TEXT, agentq TEXT NOT NULL DEFAULT 'off', runtime TEXT);
CREATE TABLE task_effects(op TEXT PRIMARY KEY, request_id INTEGER NOT NULL, project TEXT NOT NULL, board TEXT NOT NULL,
                    root TEXT NOT NULL, args TEXT NOT NULL, receipt TEXT, at INTEGER NOT NULL);
CREATE TABLE tasks(number INTEGER NOT NULL, request_id INTEGER NOT NULL, chat INTEGER NOT NULL, topic INTEGER, state TEXT NOT NULL,
                    revision INTEGER NOT NULL, card TEXT, observed INTEGER NOT NULL, closed INTEGER NOT NULL DEFAULT 0,
                    project TEXT NOT NULL DEFAULT 'pilotq', board TEXT NOT NULL DEFAULT 'taskqorg/pilotq', root TEXT NOT NULL DEFAULT '', PRIMARY KEY(project, number));
CREATE TABLE tickets(run_id INTEGER NOT NULL, idx INTEGER NOT NULL, op_id TEXT NOT NULL UNIQUE, task INTEGER, started INTEGER NOT NULL DEFAULT 0,
                    PRIMARY KEY(run_id, idx));
CREATE TABLE tool_sizes(id INTEGER PRIMARY KEY, request_id INTEGER NOT NULL, run_id INTEGER NOT NULL,
            at INTEGER NOT NULL, tool TEXT NOT NULL, result_bytes INTEGER NOT NULL, codex_version TEXT NOT NULL, pilotq_version TEXT NOT NULL);
CREATE TABLE updates(update_id INTEGER PRIMARY KEY, disposition TEXT NOT NULL, request_id INTEGER, at INTEGER NOT NULL);
CREATE INDEX tool_sizes_at ON tool_sizes(at);
PRAGMA user_version=14;
