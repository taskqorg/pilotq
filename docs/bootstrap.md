# Repository setup status

Observed on 2026-10-16. This is setup evidence, not an exception to the [specq security baseline](https://github.com/taskqorg/specq/blob/main/engineering/security.md).

- `taskqorg/pilotq` is private. The initial specification was published on `main`; GitHub displayed a successful documentation check.
- Relative document paths, local heading links and whitespace passed local validation. CI checks required document presence and whitespace only; it does not qualify product behavior.
- Dependency graph, Dependabot alerts and Dependabot security updates were enabled and read back as enabled in GitHub settings.
- **Main-branch protection is not enforced.** GitHub explicitly says private-repository rules require this organization to upgrade to Team or Enterprise. No billing change or visibility change was made. Resolve this before treating repository setup as compliant.
- Secret scanning and push protection were not exposed in the inspected private-repository settings; activation and availability remain unverified. Organization-wide two-factor enforcement and repository collaborator review remain unverified.
- The current CLI API token cannot create or read this new private repository. Repository creation used the authorized browser session; publishing used existing SSH access. API-based project automation needs separately authorized access before use.

No application, bot, production service, queue admission or runtime qualification exists yet. The project is ready for specification review, not a production claim.
