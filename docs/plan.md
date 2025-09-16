# 作業計画 (2025-09-16)

1. [x] 新機能用ブランチ `feature/auto-run-on-connect` を作成する
2. [x] 自動実行モードの要件を整理し、IOKit での接続監視方式を確定する (`docs/auto-mode-requirements.md`)
3. [x] CLI ロジックをリファクタリングして F キー切替処理を関数化する
4. [ ] IOKit コールバックを用いた常駐監視ループと `--auto` オプションを実装する
5. [ ] launchd 向け plist テンプレートと README 手順を追加する
6. [ ] 実機で接続/切断検証と CPU 負荷確認を行い、結果を記録する
