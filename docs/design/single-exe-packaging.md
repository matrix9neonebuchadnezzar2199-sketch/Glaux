# Glaux Packaging（レガシー参照）

**正本は [portable-packaging.md](portable-packaging.md) に移行しました。**

v0.2 以降、Glaux は単一 EXE 自己展開ではなく **ポータブルフォルダ配布** を採用しています。

| 旧 (v0.1) | 新 (v0.2+) |
|-----------|------------|
| `dist/Glaux.exe` 単体 | `dist/Glaux -OFFLINE AI Chat-/` フォルダ |
| EXE 内埋め込み → AppData 展開 | `runtime/` + `model/` を同階層参照 |
| E2B のみ | E2B のみ |
