# Pinned TaskQ fixtures

These files are copies of TaskQ's own contract fixtures. PilotQ's `taskq_contract_2010` test (src/taskq.rs) reads them and fails when TaskQ renames, retypes or drops a field PilotQ reads.

- Source: [taskqorg/taskq](https://github.com/taskqorg/taskq) at commit `0977146888f1d3adc4d461c51147f7234600be9d`, which includes the snapshot contract of TaskQ #2014.
- `status-snapshot.json` and `status-snapshot.schema.json`: `tests/fixtures/status-snapshot*.json` in TaskQ.
- `answer-card.json`: `tests/fixtures/answer-card.json` in TaskQ.

To update, copy the same files from a newer TaskQ commit, change the commit above, and run `cargo test --locked taskq_contract_2010`. A failure names the changed field. Change PilotQ's reader or ask TaskQ before you accept the new fixture.
