# AGENTS.md — spritore 開発エージェント向け規約

## 必読

作業前に必ず読むこと:

1. [docs/plan.md](docs/plan.md) — 設計計画の正。**「Phase 0 結果」と「ハマりどころ」は特に重要**
   (docs/ はローカル管理でリポジトリには含まれない)
2. 発注された Phase の作業指示書 (docs/phase*-*.md)

## このプロジェクトの絶対律: バイト決定論

同一入力 → 同一出力バイトを全プラットフォーム (native / wasm / Node) で保証する。

- `spritore-core` に **fs / 時刻 / 乱数 / 環境変数 / wasm-bindgen / clap を持ち込まない**
- **HashMap / HashSet の列挙順に依存しない**。列挙するコレクションは BTreeMap / BTreeSet
  を使うか、明示的にソートする
- 比較・選択のタイブレークは必ず決定的に定義する (id 昇順、フィルタ番号昇順など)
- 依存 crate は作業指示書に列挙されたもの以外追加しない (必要と思ったら理由を添えて報告)

## コーディング規約

- rustfmt (設定は rustfmt.toml、ハードタブ)。`cargo fmt --check` が通ること
- `cargo clippy --workspace -- -D warnings` が通ること
- 公開 API には doc コメント (英語) を書く。実装内コメントは日本語で良い
- コミットメッセージは `feat:` / `fix:` / `test:` / `docs:` プレフィックス

## 触ってはいけないもの

- `rust-toolchain.toml`、`Cargo.toml` の `[profile.release]`
- `npm/spritore/` は Phase 2 以降の作業指示書の範囲でのみ変更する
- `.codex/` などエージェント環境の副産物をコミットしない

## 検証コマンド

```
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
