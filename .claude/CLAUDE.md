# プロジェクト固有のルール

## Git コミット

- **Conventional Commit 形式を使用すること**
  - [commitlint-rs](https://github.com/KeisukeYamashita/commitlint-rs) による pre-commit hook で検証
  - 形式: `<type>(<scope>): <subject>`
  - type: feat, fix, docs, style, refactor, test, chore, ci, build, perf など
  - 例: `feat(cli): add new command for repository sync`
  - 例: `fix(git): resolve branch detection issue`
  - 例: `docs: update README with installation steps`

## Pull Request レビュー対応

### コメント対応後の処理
- **PR コメントへの対応が完了したら、必ず該当コメントを resolve すること**
  - GitHub CLI を使用: `gh pr comment <comment-id> --resolve`
  - または GitHub の Web UI で resolve
- 未対応・保留の場合は、その理由をコメントで明示すること

### Push 後の PR 更新
- **コミットを push した後は、必ず PR の title と description を更新すること**
  - `gh pr edit <pr-number>` を使用して更新
  - 追加した変更内容を description に反映
  - レビューコメントへの対応状況を明記
  - 更新例：
    ```bash
    gh pr edit <pr-number> --title "新しいタイトル" --body "$(cat <<'EOF'
    ## Summary
    - 変更内容のサマリー

    ## Changes
    - レビューコメントへの対応内容
    - 追加の変更

    ## Review Response
    - @reviewer のコメントに対応: XXXを修正
    EOF
    )"
    ```

### PR 作成時の注意
- title は簡潔で内容を的確に表現すること
- description には以下を含めること：
  - Summary: 変更の概要（箇条書き）
  - Test plan: テスト方法・確認項目
  - 関連する Issue や PR があれば参照を追加

## ワークフロー例

```bash
# 1. ブランチ作成
git checkout -b feature/new-functionality

# 2. 開発 & コミット（conventional commit 形式）
git commit -m "feat(module): add new functionality"

# 3. Push & PR 作成
git push -u origin feature/new-functionality
gh pr create --title "..." --body "..."

# 4. レビュー対応後
git commit -m "fix(module): address review comments"
git push

# 5. PR 更新
gh pr edit <pr-number> --body "..."

# 6. コメント resolve
gh pr comment <comment-id> --resolve
```
