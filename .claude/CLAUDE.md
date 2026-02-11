# Project Guidelines

## Pull Request Review Workflow

### Resolving Comments
- **MUST resolve PR comments after addressing them**
  - Use GitHub CLI: `gh pr comment <comment-id> --resolve`
  - Or resolve via GitHub Web UI
- If a comment cannot be addressed or is deferred, explicitly explain why in a reply

### Updating PR After Push
- **MUST update PR title and description after pushing commits**
  - Use `gh pr edit <pr-number>` to update
  - Reflect new changes in the description
  - Document review comment responses
  - Example:
    ```bash
    gh pr edit <pr-number> --title "Updated title" --body "$(cat <<'EOF'
    ## Summary
    - Change summary

    ## Changes
    - Response to review comments
    - Additional changes

    ## Review Response
    - Addressed @reviewer comment: Fixed XXX
    EOF
    )"
    ```

### PR Creation Guidelines
- Title should be concise and accurately describe the changes
- Description must include:
  - Summary: Overview of changes (bullet points)
  - Test plan: Testing methodology and verification steps
  - Related issues or PRs if applicable

## Workflow Example

```bash
# 1. Create branch
git checkout -b feature/new-functionality

# 2. Development & commit
git commit -m "feat(module): add new functionality"

# 3. Push & create PR
git push -u origin feature/new-functionality
gh pr create --title "..." --body "..."

# 4. After review feedback
git commit -m "fix(module): address review comments"
git push

# 5. Update PR
gh pr edit <pr-number> --body "..."

# 6. Resolve comments
gh pr comment <comment-id> --resolve
```
