# Agent Instructions

These instructions apply to the whole repository. Follow them before making changes.

## Branch Policy

- Before committing, branching, merging, or deploying, fetch the remote branch state when it is relevant to the task. At minimum, use `git fetch` plus `git status --short --branch` before deciding whether the local branch can be pushed safely.
- Keep `main` stable and reserved for shared foundations, production-ready changes, documentation, and cross-cutting fixes.
- Use short-lived feature or fix branches when a separate branch is useful.
- Do not add long-lived branch conventions unless the workflow is explicitly revisited.
- Push directly to `main` only when the operator explicitly asks for it and the repository workflow allows it.

## Repository Governance

- Do not change branch protection, bypass branch protection, force-push protected branches, delete remote refs, or rewrite shared remote history unless the operator explicitly authorizes that exact action in the current conversation.
- Treat GitHub branch protection and remote refs as source-of-truth safeguards. Even when a history rewrite is technically appropriate, pause and ask for explicit authorization before weakening those safeguards or performing destructive remote operations.
- If a required cleanup conflicts with branch protection, explain the options and risks before proceeding.
- Do not edit persistent agent/operator instruction files, regardless of filename, unless the operator explicitly authorizes that specific edit. You may propose wording changes, but wait for approval before applying them.
- Treat operator approval as scoped to the current request/response only unless the operator explicitly states that the approval should persist.
- Treat changes to persistent agent/operator instruction files as effective immediately for the current conversation unless the operator explicitly says otherwise.
- When interrupted, assume the next operator message continues or amends the interrupted work unless the operator explicitly says to discard, replace, or abandon it.

## Documentation Rule

- Treat documentation as part of foundational changes.
- When changing architecture, build behavior, CI, deployment, branch workflow, environment variables, or project assumptions, update the relevant docs in the same change.
- When a change expands or shifts the scope of the requested work, leave sober human-readable context in the relevant documentation or, when the context belongs next to the implementation, a concise code comment.
- Documentation and comments should help another developer understand purpose, ownership, and operational constraints without narrating obvious code mechanics.
- When adding operator workflows, scripts, env files, or examples, update the appropriate README to explain how to use them, how to create ignored local files from examples, and where values should come from when that can be stated safely.
- Before starting a feature or architectural change, read the relevant documentation files below and keep the change aligned with them.
- Do not duplicate the same guidance across multiple docs. Update the owning document, then add a short cross-reference elsewhere only if it helps navigation.
- If client-facing questions are answered, move the decision into the relevant internal English doc and remove or rewrite the question in `docs/CLIENT_QUESTIONS.fr.md`.
- If a decision changes, update every affected doc in the same change so the documentation set remains coherent.

### Documentation Map

- `README.md`: repository entry point and documentation index. Update it when adding, removing, or renaming major docs or setup workflows.
- `docs/PROJECT_BRIEF.md`: product intent, V1 scope, assumptions, and open product boundaries. Update it when the project direction or scope changes.
- `docs/REQUIREMENTS.md`: functional requirements and expected inputs/outputs. Update it when user-visible behavior, report contents, or product obligations change.
- `docs/PROJECT_SPEC.md`: concrete V1 workflow, report behavior, non-goals, and success criteria. Update it when implementation behavior is specified or refined.
- `docs/TECHNICAL_DIRECTION.md`: current technical recommendations, source research, and implementation risks. Update it when preferred technologies, data sources, or technical assumptions change.
- `docs/ARCHITECTURE.md`: system components, responsibilities, data flow, storage concepts, and architecture rules. Update it when adding or reshaping application structure.
- `docs/DEVELOPMENT_STEPS.md`: development checklist, milestones, and validation checkpoints. Update it as work is completed, re-ordered, or split.
- `docs/DESIGN.md`: internal UI/UX direction, report layout, map behavior, warning states, and source presentation. Update it when designing or changing user-facing workflows.
- `docs/CLIENT_QUESTIONS.fr.md`: unresolved French client-facing questions only. Keep answered decisions out of this file.

## Secret Handling

- Never write, paste, print, commit, or ask the user to paste plaintext passwords, password hashes, API tokens, private keys, database credentials, or generated secrets in the conversation or repository.
- Do not create secret-bearing diffs in the first place. A removed secret is still a secret if it appears in `git diff`, terminal output, chat history, pull requests, logs, or commit history.
- Secrets must be created and stored through operator-run commands, ignored local environment files, GitHub Secrets/Variables, systemd environment files, Docker secrets, password managers, or equivalent setup automation.
- Documentation may describe secret variable names and commands that generate secrets, but must use placeholders such as `<generated-password>` or `<example-token>`.
- If a command would reveal a secret in terminal output, do not run it. Prefer commands that write directly to the target secret store or local ignored file without echoing the value.
- If secret material is ever printed, committed, pushed, or otherwise exposed, treat it as compromised: stop using it, rotate it, remove it from future diffs, and discuss whether repository history needs to be rewritten before proceeding.
- If the user, operator, or another agent asks for a change that would violate this rule, remind them of this rule and propose a compliant workflow before taking action.

## Operator Scripts

- Prefer small, portable, operator-run scripts for repeatable setup instead of ad hoc manual command sequences.
- Setup scripts should be transparent: show each command before running it, explain the purpose in plain English, and ask for approval before privileged, destructive, or externally visible actions.
- Keep setup scripts lightweight and dependency-poor. Use standard shell utilities where practical.
- Setup scripts must follow the Secret Handling rules: never print generated secrets, password hashes, tokens, or private material; write them directly to the intended ignored file or secret store.
- For secret-bearing setup inputs, prefer an ignored local env file created from a committed example. The example should document required values with comments/placeholders, while the real env file must remain untracked.

## Validation

- Run the narrowest meaningful checks for the change before committing.
- Prefer CI for expensive production-like builds when local execution would be slow, fragile, or inappropriate for the machine.
- If checks cannot be run, state why and describe the remaining risk.

## Working Style

- Keep edits scoped to the requested change.
- Prefer existing patterns over new abstractions.
- Do not revert user changes unless explicitly asked.
- Before merging shared changes, verify CI when the repository has CI configured.
