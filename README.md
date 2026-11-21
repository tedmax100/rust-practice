# Rust Practice & Codelabs

[![Deploy Codelabs to GitHub Pages](https://github.com/tedmax100/rust-practice/actions/workflows/deploy-codelabs.yml/badge.svg)](https://github.com/tedmax100/rust-practice/actions/workflows/deploy-codelabs.yml)
[![Security](https://github.com/tedmax100/rust-practice/actions/workflows/security.yml/badge.svg)](https://github.com/tedmax100/rust-practice/actions/workflows/security.yml)

Rust 實戰與進階教學系列，所有教學範例皆嚴格遵循 [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)，並透過 CI 進行自動化格式檢查、靜態分析（Clippy）與單元測試。

---

## 🌐 線上閱讀（GitHub Pages）

教學網站已部署至 GitHub Pages，支援深淺色主題切換與程式碼語法高亮：

🔗 **教學網站首頁入口**：[https://tedmax100.github.io/rust-practice/](https://tedmax100.github.io/rust-practice/)

### 📚 教學課程清單與直達連結

| 課程 | 說明 | 直達連結 |
|---|---|---|
| **Rust 入門：從第一行就寫對** | 對照 Google Rust Style Guide，養成良好慣用法。涵蓋變數、所有權、struct/enum、錯誤處理、trait 與測試文件。 | [進入課程 ➔](https://tedmax100.github.io/rust-practice/rust-style-basics/) |
| **Go 開發者學 Rust：從 go mod 到 Cargo** | 給熟悉 Go 的開發者：用 go mod、interface、goroutine 視角切入 Rust，全範例附 Go 對照。 | [進入課程 ➔](https://tedmax100.github.io/rust-practice/rust-for-gophers/) |
| **安全的代價：揭秘 Rust 效能（一）** | 探討記憶體安全代價、動態邊界檢查與自動向量化瓶頸，以及零 unsafe 榨乾 CPU 的五大實戰技巧。 | [進入課程 ➔](https://tedmax100.github.io/rust-practice/rust-performance-safety/) |
| **Rust 效能提升技巧與慣用法（二）** | 寫出編譯器最愛的零成本抽象：迭代器消除邊界檢查、裸切片別名分析、Buffer 重用與結構體記憶體重排。 | [進入課程 ➔](https://tedmax100.github.io/rust-practice/rust-performance-idioms/) |

---

## 🛠 專案特色

- **零違規保證**：每一段教學範例代碼均在 CI 中透過 `rustfmt`、`clippy-driver -D warnings` 與 `rustc --test` 自動測試。
- **Google Codelabs 體驗**：使用 Google 官方 `claat` 工具建置步驟式教學網站，並套用 DevSite 現代化主題與程式碼複製擴充。
- **供應鏈安全**：內建 `cargo audit`、`cargo deny` 與 `cargo machete`，自動防範有漏洞或未經授權的相依套件。
- **跨平台建置與容器化**：支援 Linux（glibc / musl）、macOS（Apple Silicon）、Windows 跨平台二進位檔案產出，並附極簡 Scratch / Distroless Dockerfile。

---

## 💻 本機開發與預覽

### 1. 環境需求

- Rust 1.80+ 工具鏈（含 `rustfmt`、`clippy`）
- Python 3
- [claat (Google Codelabs CLI)](https://github.com/googlecodelabs/tools)

### 2. 本機建置教學站

```bash
# 建置教學網站（含 Callout lint、Snippet 測試與 claat 匯出）
./scripts/build_site.sh
```

### 3. 本機預覽

```bash
# 預設啟動於 http://localhost:8088
./scripts/serve.sh
```

### 4. 執行專案風格與測試檢查

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
