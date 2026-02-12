# Project Guidelines

## Pull Request Best Practices

### PR Size and Scope

- **Keep PRs small and focused**
  - Easier to review and less likely to introduce bugs
  - Aim for <400 lines of changes when possible
  - Split large features into multiple PRs
- **Single responsibility**: One PR should address one concern

### Before Creating PR

- **Self-review required**
  - Review your own changes before requesting review
  - Check for debug code, commented code, or TODO items
  - Verify code style consistency
- **CI/Tests must pass**
  - Ensure all automated checks pass
  - Run tests locally before pushing
  - Fix linting and formatting issues

### PR Creation Guidelines

- **Title**: Concise and accurately describe the changes
- **Description language**: Write in English for consistency and broader accessibility
- **Use the PR template**: Fill out all sections in `.github/pull_request_template.md`
  - **Summary**: Overview of changes (bullet points)
  - **Changes**: Detailed explanations
  - **Test Plan**: Verification steps (checkboxes)
  - **Breaking Changes**: Document any breaking changes
  - **Related Issues/PRs**: Link if applicable
- **Request Copilot review**: Use GitHub Copilot to get automated feedback

### Updating PR After Push

- **MUST update PR description after pushing new commits**
  - Edit the PR description on GitHub web UI or use `gh pr edit <pr-number>`
  - Reflect new changes in the description
  - Document review comment responses
  - Add a "Review Response" section to explain how you addressed comments

### Comment Resolution

- Comments are automatically enforced by GitHub's "Require conversation resolution before merging" setting
- Always reply to comments explaining your changes or reasoning

## Workflow Example

**Note**: Commit messages follow Conventional Commits format, enforced by commitlint-rs git hook.

```bash
# 1. Create branch
git checkout -b feature/new-functionality

# 2. Development & commit (format enforced by git hook)
git commit -m "feat(module): add new functionality"

# 3. Self-review before creating PR
git diff main...HEAD
# Review changes, check for debug code, verify tests

# 4. Push & create PR
git push -u origin feature/new-functionality
gh pr create --title "..." --body "..."

# 5. Request Copilot review (via GitHub UI)

# 6. After receiving review feedback (format enforced by git hook)
git commit -m "fix(module): address review comments"
git push

# 7. Update PR description
gh pr edit <pr-number> --body "..."

# 8. Reply to review comments explaining changes
# 9. Resolve conversations (enforced by GitHub settings)
```
