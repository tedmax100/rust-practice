author: Nathan
summary: 從零開始學 Rust，每一章都對照 Google Rust Style Guide，從第一行程式就養成好習慣
id: rust-style-basics
categories: rust,style-guide,beginner
environments: Web
status: Published

# Rust 入門：從第一行就寫對（對照 Google Rust Style Guide）

## 課程簡介與環境
Duration: 5

### 歡迎來到 Rust！

這門課用 Codelabs 的步驟式格式，帶你從零學 Rust。和一般教學不同的是：**每一章的範例都對照 [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)**，讓你不用事後改壞習慣。

### 你將學到什麼

- 變數、型別、函式與控制流程
- 所有權與借用（Rust 最重要的觀念）
- struct、enum 與 newtype 的設計
- `Option` / `Result` 錯誤處理
- trait、迭代器與命名慣例
- 模組、文件註解與測試

### 如何保證教學本身沒有違反風格指南？

本專案的每一段 Rust 範例都會在建置時被自動檢查：

| 檢查 | 工具 | 對應指南內容 |
|------|------|--------------|
| 格式 | `rustfmt`（`max_width = 100`） | Code formatting |
| 慣用寫法與禁用項目 | `clippy`（見 `Cargo.toml` 的 `[lints]`） | Ownership、Function inputs、Implementation concerns |
| 可編譯、測試通過 | `rustc --test` | Test frameworks |

Positive
: 刻意示範錯誤寫法的區塊，第一行會標註「反例」，建置檢查會略過它們，其餘範例必須通過全部檢查。

### 安裝環境

安裝 Rust 工具鏈（含 `cargo`、`rustfmt`、`clippy`）：

```console
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add rustfmt clippy
cargo --version
```

### 專案的風格設定檔

專案根目錄的 `rustfmt.toml`：

```toml
max_width = 100
```

之後每次提交前執行：

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Negative
: 風格指南的 `imports_granularity = "Module"`、`hex_literal_case = "Lower"` 等 rustfmt 選項目前需要 nightly 工具鏈，stable 無法啟用，本課程以人工遵守並在對應章節提醒。

## 第一個程式與 Cargo
Duration: 5

### Hello, world

```rust
fn main() {
    println!("Hello, world!");
}
```

用 `cargo new hello` 建立專案，`cargo run` 執行（下一章會詳細介紹 Cargo）。`fn main` 是程式進入點，`println!` 結尾的 `!` 表示它是巨集。

### 風格指南怎麼說？

- 格式一律交給 `rustfmt`，不要手動調整。需要 `#[rustfmt::skip]` 只能是為了刻意對齊或產生的程式碼，不可因為個人喜好。
- 巨集的優先順序：先用一般函式、型別、trait；其次才是 `macro_rules!`；`proc_macro` 是最後手段。初學階段只要會「使用」巨集就夠了。

## Cargo 與專案結構
Duration: 10

Cargo 是 Rust 的建置工具與套件管理員：建立專案、編譯、執行、測試、下載依賴都靠它。

### 建立專案

```console
cargo new hello
cd hello
cargo run
```

`cargo new` 會產生下面的結構：

```text
hello/
├── Cargo.toml      專案設定（名稱、版本、依賴、lint）
├── .gitignore      預設忽略 /target
└── src/
    └── main.rs     程式進入點
```

第一次 `cargo run` 之後還會多出：

- `Cargo.lock`：記錄實際解析出的依賴版本。
- `target/`：編譯產物，已被 `.gitignore` 忽略，不要提交。

Positive
: 建立函式庫用 `cargo new --lib mylib`，入口檔會變成 `src/lib.rs`。

### 逐行讀懂 Cargo.toml

本專案的設定如下：

```toml
[package]
name = "rust_practice"
version = "0.1.0"
edition = "2021"

[dependencies]

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
unwrap_used = "warn"
wildcard_imports = "warn"
```

| 區塊 | 意義 |
|------|------|
| `[package]` | 專案名稱、版本；`edition` 是 Rust 語言版本，新專案用 `2021`（或更新的版本） |
| `[dependencies]` | 執行時需要的外部套件（crate） |
| `[lints.rust]` / `[lints.clippy]` | 專案層級的 lint 規則，需要 Rust 1.74 以上 |

把 lint 寫在 `Cargo.toml`，團隊每個人與 CI 就用同一組規則，這正是用工具落實風格指南的方式。

### 常用指令

| 指令 | 用途 | 使用時機 |
|------|------|----------|
| `cargo check` | 只檢查能否編譯，不產生執行檔 | 編輯時最常用，速度最快 |
| `cargo build` | 編譯 debug 版 | 需要執行檔時 |
| `cargo run` | 編譯並執行 | 日常開發 |
| `cargo build --release` | 編譯最佳化版本 | 發布、效能測試 |
| `cargo test` | 執行測試 | 提交前 |
| `cargo fmt` | 用 rustfmt 格式化 | 存檔或提交前 |
| `cargo clippy` | 靜態分析 | 提交前 |
| `cargo doc --open` | 產生並開啟文件 | 查 API |

debug 與 release 的產物分別放在 `target/debug/` 與 `target/release/`。

### 加入依賴

```console
cargo add rand@0.8
```

這會在 `Cargo.toml` 的 `[dependencies]` 加上：

```toml
[dependencies]
rand = "0.8"
```

之後 `cargo build` 會自動從 crates.io 下載並編譯。`"0.8"` 代表相容 0.8.x 的版本，實際解析到的精確版本記錄在 `Cargo.lock`。

Negative
: 依賴要有明確的版本需求，不要寫成 `rand = "*"`。`*` 會讓任何版本都符合，之後一次更新就可能讓整個專案無法編譯。

### Cargo.lock 要不要提交？

- **應用程式（binary）**：要提交，確保每個人與 CI 建置出相同結果。
- **函式庫（library）**：傳統上可不提交，因為使用者會自行解析版本。

本專案是應用程式，所以提交 `Cargo.lock`。

### 風格指南相關重點

- 格式化永遠交給 `cargo fmt`；`rustfmt.toml` 放在專案根目錄，所有人共用。
- 風格指南偏好**小而專注的 crate**，一個 crate 負責一件事；專案變大時用 workspace 把多個 crate 放在同一個 repo。
- 第三方依賴要審慎：每多一個依賴就多一份維護與安全責任。
- 提交前固定跑 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test`，這三個指令就是風格指南的自動把關。

## 變數、型別與函式
Duration: 8

### 變數預設不可變

```rust
fn main() {
    let language = "Rust";
    let mut score = 0;
    for round in 1..=3 {
        score += round * 10;
    }
    println!("{language} 得分：{score}");
}
```

- `let` 宣告不可變變數；需要修改才加 `mut`。
- `1..=3` 是包含 3 的範圍。
- `{language}` 是行內格式化引數，把變數直接寫進字串。

### 函式與回傳值

```rust
fn add(left: i32, right: i32) -> i32 {
    left + right
}

fn main() {
    let total = add(2, 3);
    println!("2 + 3 = {total}");
}
```

函式最後一個**沒有分號**的運算式就是回傳值。

### 型別標註

Positive
: 風格指南建議：型別推論不明顯時就寫出明確型別，並且偏好型別標註而不是 turbofish（`::<>`）。

```rust
fn main() {
    let numbers: Vec<i32> = "1,2,3"
        .split(',')
        .map(|text| text.parse().unwrap_or(0))
        .collect();
    println!("{numbers:?}");
}
```

上面用 `let numbers: Vec<i32>` 標註，而不是 `.collect::<Vec<i32>>()`。

## 所有權與借用
Duration: 12

### 三條規則

1. 每個值都有一個擁有者。
2. 同一時間只有一個擁有者。
3. 擁有者離開作用域，值就被釋放。

### 借用：不拿走所有權

```rust
fn char_count(text: &str) -> usize {
    text.chars().count()
}

fn main() {
    let owned = String::from("你好，Rust");
    println!("{}", char_count(&owned));
    println!("{}", char_count("字串常值"));
    println!("{owned}");
}
```

`char_count(&owned)` 只是借用，呼叫完 `owned` 依然可用。

### 風格指南：函式參數要用最通用的借用型別

| 不要 | 改用 |
|------|------|
| `&String` | `&str` |
| `&Vec<T>` | `&[T]` |
| `&Box<T>` | `&T` |
| `&Option<T>` | `Option<&T>` |

```rust
// 反例：只能接受 String，無法傳入字串常值
fn char_count(text: &String) -> usize {
    text.chars().count()
}

fn main() {
    println!("{}", char_count(&String::from("abc")));
}
```

Negative
: clippy 的 `ptr_arg` 會直接警告這種寫法，所以不用硬背，讓工具提醒你。

### 風格指南：不要用 clone() 逃避借用

優先使用唯一所有權與借用，`Rc` / `Arc` 只有在替代方案過於複雜時才使用。`.clone()` 應該只在你真的需要一份深拷貝時使用。

```rust
fn longest<'a>(first: &'a str, second: &'a str) -> &'a str {
    if first.len() >= second.len() {
        first
    } else {
        second
    }
}

fn main() {
    let first = String::from("borrow");
    let second = String::from("checker");
    let result = longest(&first, &second);
    println!("較長的是 {result}");
}
```

這裡完全不需要 `clone()`：用借用加上生命週期標註就能回傳參考。

## struct、enum 與 newtype
Duration: 12

### 用 enum 取代意義不明的 bool

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Post {
    title: String,
    visibility: Visibility,
}

impl Post {
    fn new(title: &str, visibility: Visibility) -> Self {
        Self {
            title: title.to_string(),
            visibility,
        }
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn is_public(&self) -> bool {
        self.visibility == Visibility::Public
    }
}

fn main() {
    let posts = [
        Post::new("Hello Rust", Visibility::Public),
        Post::new("草稿", Visibility::Private),
    ];
    for post in &posts {
        println!("{} 是否公開：{}", post.title(), post.is_public());
    }
}
```

對照反例：`Post::new("Hello Rust", true)` 讀的人不知道 `true` 是什麼意思。風格指南建議用 enum；若真的保留 `bool`，至少加上行內註解，例如 `Post::new("Hello", /* public= */ true)`。

### 風格指南重點

- **用 `#[derive()]`**：能 derive 的 trait 就不要手寫。
- **優先具名欄位**：避免用 tuple 或一般的 tuple struct，唯一例外是單一欄位的 newtype。
- **建構函式**用 `new()`；有多種建構方式時用 `new_with_foo()` 或 `from_foo()`；可能失敗則用 `try_new()` 回傳 `Result`。
- **getter**：`foo()` 回傳參考（`Copy` 型別回傳值）、`foo_mut()`、`set_foo()`，不要加 `get_` 前綴。

### newtype：避免混用相同的底層型別

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OrderId(u64);

fn describe_order(user: UserId, order: OrderId) -> String {
    format!("使用者 {} 的訂單 {}", user.0, order.0)
}

fn main() {
    let message = describe_order(UserId(7), OrderId(1001));
    println!("{message}");
}
```

如果兩個參數都是 `u64`，編譯器就無法阻止你把順序傳反；用 newtype 而不是 `type` 別名，錯誤會在編譯期就被抓到。

## 錯誤處理：Option 與 Result
Duration: 10

### 何時用哪一個？

| 情況 | 使用 |
|------|------|
| 簡單、明顯的「沒有值」 | `Option` |
| 呼叫端需要分辨的可恢復錯誤（I/O、網路、解析） | `Result` |
| 程式 bug、違反不變條件 | `panic!` |

### 範例

```rust
use std::num::ParseIntError;

fn parse_port(text: &str) -> Result<u16, ParseIntError> {
    text.trim().parse()
}

fn first_even(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().find(|number| number % 2 == 0)
}

fn describe(numbers: &[i32]) -> String {
    let Some(value) = first_even(numbers) else {
        return String::from("沒有偶數");
    };
    format!("第一個偶數是 {value}")
}

fn main() {
    match parse_port("8080") {
        Ok(port) => println!("port = {port}"),
        Err(error) => println!("無效的 port：{error}"),
    }
    println!("{}", describe(&[1, 3, 4]));
    let fallback: u16 = "443".parse().expect("內建常數必定是合法的 port");
    println!("{fallback}");
}
```

### 風格指南重點

- 優先使用 `match`、`if let`、`let ... else`，而不是「先檢查再 `unwrap()`」。
- 用 `expect("原因")` 取代 `unwrap()`，讓 panic 訊息說明為什麼這裡不可能失敗。Cargo.toml 已開啟 `clippy::unwrap_used`。
- `assert!` 要附上說明預期的訊息。
- 絕對不要在 first-party 程式碼用 `catch_unwind` 當作正常錯誤處理。

## trait 與迭代器
Duration: 12

### 實作標準 trait，而不是自創方法

如果方法的意義符合標準 trait，就實作該 trait：轉字串用 `FromStr`，顯示用 `Display`。

```rust
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Celsius(i32);

impl fmt::Display for Celsius {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}°C", self.0)
    }
}

impl FromStr for Celsius {
    type Err = ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let degrees = text.trim_end_matches("°C").trim().parse()?;
        Ok(Self(degrees))
    }
}

fn main() {
    match "25°C".parse::<Celsius>() {
        Ok(temperature) => println!("解析成功：{temperature}"),
        Err(error) => println!("解析失敗：{error}"),
    }
}
```

這裡用 `use std::fmt;` 後寫 `fmt::Display`：風格指南建議「一個 `use` 只匯入一個模組」、不使用 `use x::*`，且對 top-level 函式透過模組呼叫。

### 迭代器

```rust
fn even_squares(numbers: &[i32]) -> impl Iterator<Item = i32> + '_ {
    numbers
        .iter()
        .filter(|number| *number % 2 == 0)
        .map(|number| number * number)
}

fn main() {
    let numbers = [1, 2, 3, 4, 5, 6];
    let squares: Vec<i32> = even_squares(&numbers).collect();
    println!("{squares:?}");
}
```

風格指南對迭代器的建議：

- **library 程式碼**盡量回傳迭代器（`impl Iterator`）而不是 `Vec`；應用程式碼為了簡單可以直接回傳 `Vec`。
- 避免過長的鏈式呼叫，尤其不要把長鏈放在 `for` 的標頭；先指派給變數。
- 沒有充分理由不要用 `for_each`，用 `for` 迴圈。

### trait 的使用時機

只有三種情況才定義 trait：為了泛型、為了接受或回傳 `impl` / `dyn Trait`，或是 extension trait（名稱以 `Ext` 結尾）。**不要只為了組織程式碼而定義 trait**，那是 inherent `impl` 的工作。

## 模組、文件與測試
Duration: 10

### 完整範例

```rust
//! 簡單的溫度換算工具。

/// 將攝氏溫度換算為華氏溫度。
///
/// 公式為 `celsius * 9 / 5 + 32`，此函式不會 panic。
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

fn main() {
    println!("100°C = {}°F", celsius_to_fahrenheit(100.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freezing_point_matches() {
        let fahrenheit = celsius_to_fahrenheit(0.0);
        assert!(
            (fahrenheit - 32.0).abs() < 1e-9,
            "0°C 應該等於 32°F，實際為 {fahrenheit}"
        );
    }
}
```

### 風格指南重點

- **文件**：`///` 用於項目、`//!` 用於模組與 crate，`//` 用於實作備註與 TODO。公開 API 都應該有文件；不要解釋基本語言概念，也不要寫只重述名稱的註解。
- **文件要寫明** panic 與錯誤條件（clippy 的 `missing_panics_doc`、`missing_errors_doc` 會檢查）。
- **模組**：檔名與 leaf 模組以主要型別命名（`Foo` 放在 `foo.rs`）；非 leaf 模組只放 `mod` 宣告與 re-export。
- **`use super::*`** 只在測試模組這類 inline 模組中允許，其他地方禁用 glob import。
- **測試**：使用專案指定的框架；一般專案以內建 `#[test]`。`assert!` 要附說明訊息。

## 風格指南檢查清單
Duration: 5

### 一頁速查

| 主題 | 做法 | 工具檢查 |
|------|------|----------|
| 格式 | 交給 `rustfmt`，`max_width = 100` | `cargo fmt --check` |
| 匯入 | 不用 glob；一個 `use` 一個模組 | `clippy::wildcard_imports` |
| 參數型別 | `&str`、`&[T]`、`Option<&T>` | `clippy::ptr_arg` |
| 所有權 | 借用優先，不亂 `clone()` | `clippy::redundant_clone` |
| bool 參數 | 意義不明時改用 enum | `clippy::fn_params_excessive_bools` |
| 型別 | 不同語意用 newtype；優先具名欄位 | 人工審查 |
| 錯誤 | `Result` / `Option`；`expect("原因")` | `clippy::unwrap_used` |
| trait | 標準 trait 優先；`#[derive]` | 人工審查 |
| 文件 | 公開 API 有文件，註明 panic 與錯誤 | `clippy::missing_*_doc` |
| unsafe | 避免；必要時加 `SAFETY:` 註解 | `unsafe_code = "forbid"` |

### 提交前的一鍵檢查

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
python3 scripts/check_snippets.py codelabs/tutorials/*.md
```

### 延伸閱讀

- [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Comprehensive Rust（Google）](https://google.github.io/comprehensive-rust/)

Positive
: 完成！你已經看過一遍 Rust 的核心觀念，並且知道每一個觀念在業界風格指南裡對應的要求。
