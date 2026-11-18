author: Nathan
summary: 給熟悉 Go 的開發者：用 go mod、interface、goroutine 的視角學 Rust，並遵守 Google Rust Style Guide
id: rust-for-gophers
categories: rust,go,style-guide
environments: Web
status: Published

# Go 開發者學 Rust：從 go mod 到 Cargo

## 課程簡介
Duration: 5

### 這門課為誰而寫

你已經很熟 Go：知道 `go mod`、interface、goroutine、`if err != nil`。這門課用你熟悉的概念當橋梁，**每個主題都先放 Go 寫法，再放對應的 Rust 寫法**，並說明為什麼兩邊的設計不同。

### 兩個語言的核心差異

| 面向 | Go | Rust |
|------|----|------|
| 記憶體 | 垃圾回收（GC） | 所有權與借用，編譯期決定何時釋放 |
| 空值 | `nil`、零值 | 沒有 `nil`，用 `Option<T>` |
| 錯誤 | `(T, error)` | `Result<T, E>` 加 `?` |
| 抽象 | interface（隱式實作） | trait（明確 `impl`） |
| 並發 | goroutine、channel | thread、channel、async（需要 crate） |
| 工具鏈 | `go` 指令 | `cargo` 指令 |
| 風格 | gofmt 統一、風格爭議少 | rustfmt + clippy，並有 Google Rust Style Guide |

### 本課程怎麼保證範例正確

建置時每一段範例都會自動檢查：

- Go 範例：`gofmt` 與 `go vet`。
- Rust 範例：`rustfmt`（`max_width = 100`）、`clippy -D warnings` 與 `rustc --test`，並對照 [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)。

Positive
: 如果你還沒有 Rust 的基礎語法，可以先看姊妹課程「Rust 入門：從第一行就寫對」，這門課則專注在 Go 開發者最需要轉換的觀念。

## Cargo 與 go mod
Duration: 12

### 一張對照表

Cargo 同時扮演 `go mod`、`go build`、`go test`、`go fmt`、`go vet`、`go doc` 與 `go install`。

| Go | Rust | 說明 |
|----|------|------|
| `go.mod` | `Cargo.toml` | 專案名稱與依賴宣告 |
| `go.sum` | `Cargo.lock` | 見下方說明，作用不完全相同 |
| `go get example.com/x` | `cargo add x` | 從 crates.io 取得套件 |
| `go get -u` | `cargo update` | 更新到相容範圍內的最新版 |
| `go mod tidy` | `cargo build`（自動維護 lock） | 找出未使用的依賴需另外安裝 `cargo-machete` |
| `go mod vendor` | `cargo vendor` | 把依賴複製進專案 |
| `go mod why x` | `cargo tree -i x` | 為什麼依賴了某套件 |
| `go list -m all` | `cargo tree` | 依賴樹 |
| `go work` | workspace | 多個 module（crate）放在一個 repo |
| `replace` | `[patch]` 或 `path = "..."` | 以本機或其他來源取代依賴 |
| `go build` / `go run` / `go test` | `cargo build` / `cargo run` / `cargo test` | 用法幾乎相同 |
| `gofmt` | `rustfmt`（`cargo fmt`） | 統一格式 |
| `go vet` / golangci-lint | `clippy`（`cargo clippy`） | clippy 內建，規則更多 |
| `go install x@latest` | `cargo install x` | 安裝執行檔 |
| pkg.go.dev | docs.rs 與 crates.io | 文件與套件搜尋 |

### 並排看設定檔

Go 的 `go.mod`：

```text
module example.com/hello

go 1.22

require github.com/google/uuid v1.6.0
```

Rust 的 `Cargo.toml`：

```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2021"

[dependencies]
uuid = "1.6"
```

差異：

- Go 的 module 路徑是 URL（`example.com/hello`），Rust 的 crate 名稱是 crates.io 上註冊的名稱（`uuid`）。
- `edition = "2021"` 類似 `go 1.22` 這一行：決定語言版本。不同 edition 的 crate 可以互相依賴。
- Rust 的 `version` 是 crate 自己的版本，Go 的版本由 git tag 決定。

### 版本選擇：MVS 與 Cargo 的差別

- **Go（MVS，最小版本選擇）**：選符合所有需求中的**最小**可用版本，`go.mod` 本身就能重現建置。`go.sum` 只存校驗碼，用來驗證下載內容。
- **Cargo**：`"1.6"` 的意思是 `^1.6`，即 `>=1.6.0, <2.0.0`，會選範圍內**最新**的版本，再由 `Cargo.lock` 記錄實際解析的每一個版本。

所以 `Cargo.lock` 比較像「完整的版本快照」，角色更接近 `go.mod` 加 `go.sum` 的鎖定部分。

Negative
: 應用程式要提交 Cargo.lock 才能重現建置，這點和 Go 不同：Go 的 go.mod 已經足以重現。

### 同一個 crate 的多個主要版本

Go 的主要版本放在 import 路徑（`/v2`）。Cargo 則允許同一個 crate 的不同**不相容**主要版本同時存在於依賴樹，各自編譯，不需要改路徑。

### features：對應 build tag

Go 用 `//go:build` tag 做條件編譯。Cargo 用 **features**，由使用者在依賴宣告時挑選：

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
```

程式碼裡用 `#[cfg(feature = "...")]` 控制是否編譯。

### 其他對照

- `go generate` 對應 `build.rs`：編譯前執行的腳本。
- 私有模組（`GOPRIVATE`）對應 git 依賴 `foo = { git = "https://..." }` 或私有 registry。
- 弱點掃描：`govulncheck` 對應 `cargo audit`（需另外安裝）。
- workspace 的設定與使用見姊妹課程的 Workspace 章節。

### 風格指南相關

- 依賴要有明確版本，不要寫 `"*"`。
- 風格指南偏好**小而專注的 crate**，這與 Go 的小 package 精神相近。
- 專案層級的 lint 寫在 `Cargo.toml` 的 `[lints]`，等同於把 `golangci-lint` 設定放進 repo。

## 型別、零值與轉型
Duration: 10

### 沒有「隱式零值」

Go 的每個變數都有零值，struct 欄位沒填就是零值：

```go
package main

import "fmt"

type Config struct {
	Host    string
	Port    int
	Verbose bool
}

func main() {
	config := Config{Port: 8080}
	fmt.Printf("%q:%d verbose=%v\n", config.Host, config.Port, config.Verbose)
}
```

Rust 的 struct 每個欄位都必須初始化。想要「其餘用預設值」要明確使用 `Default`：

```rust
#[derive(Debug, Default)]
struct Config {
    host: String,
    port: u16,
    verbose: bool,
}

fn main() {
    let config = Config {
        port: 8080,
        ..Default::default()
    };
    println!(
        "{:?}:{} verbose={}",
        config.host, config.port, config.verbose
    );
}
```

`..Default::default()` 的意思是：其餘欄位取自 `Default` 的結果。

Positive
: 風格指南建議：有 new() 或明確的預設狀態時就實作 Default，所有欄位都公開的「選項包」struct 也應該實作 Default，這正是 Go 零值慣例的 Rust 版本。

### 變數：`:=` 與 `let`

| Go | Rust |
|----|------|
| `x := 1`（可變） | `let mut x = 1;` |
| 沒有常數變數，用 `const` | `let x = 1;` 預設不可變 |
| 遮蔽（同名重新宣告） | `let x = x * 2;` 是明確的遮蔽 |

在 Rust，**預設不可變**，要修改才寫 `mut`。

### 數字轉型：不會悄悄溢位

Go 的轉型是無條件的，窄化時會靜默截斷：

```go
package main

import "fmt"

func main() {
	big := 300
	narrow := uint8(big)
	fmt.Println(narrow)
}
```

這會印出 `44`。Rust 不允許隱式轉型，並提供安全的轉換：

```rust
fn main() {
    let small: i32 = 42;
    let wide = i64::from(small);
    let checked = u8::try_from(300_i32);
    println!("{wide} {checked:?}");
}
```

- `i64::from(small)`：不會失敗的放寬轉換，用 `From`。
- `u8::try_from(300)`：可能失敗的轉換，回傳 `Result`，這裡會是 `Err`。

Negative
: Rust 也有 as 轉型，行為與 Go 的強制轉型類似，會靜默截斷。風格指南與 clippy 都建議盡量用 From 與 TryFrom，只有真的需要位元層級的行為才用 as。

## 所有權：沒有 GC 的記憶體管理
Duration: 15

這是 Go 開發者最大的思維轉換。Go 用 GC 自動回收，你幾乎不用想記憶體；Rust 沒有 GC，由**編譯器**根據所有權規則決定何時釋放，執行期零成本。

### 三條規則

1. 每個值有一個擁有者。
2. 同一時間只有一個擁有者。
3. 擁有者離開作用域，值就被釋放。

### 賦值是「移動」，不是「共享」

Go 的 slice、map 賦值後共用底層資料：

```go
package main

import "fmt"

func main() {
	original := []int{1, 2, 3}
	alias := original
	alias[0] = 99
	fmt.Println(original[0])
}
```

印出 `99`：兩個變數指向同一塊資料。Rust 的賦值會**移動**所有權，舊變數不能再用：

```rust
fn consume(text: String) -> usize {
    text.len()
}

fn main() {
    let text = String::from("hello");
    let length = consume(text);
    // println!("{text}"); // 編譯錯誤：text 的所有權已移入 consume
    println!("{length}");
}
```

### 借用：`&` 與 `&mut`

不想交出所有權，就**借用**。`&T` 是唯讀借用，可以同時存在多個；`&mut T` 是獨佔的可變借用，同一時間只能有一個，而且不能與 `&T` 並存。

```rust
fn push_one(items: &mut Vec<i32>) {
    items.push(1);
}

fn total(items: &[i32]) -> i32 {
    items.iter().sum()
}

fn main() {
    let mut items = vec![10, 20];
    push_one(&mut items);
    println!("總和 = {}", total(&items));
}
```

這條規則在編譯期就防止資料競爭與迭代中修改集合，等於把 Go 的 `go test -race` 搬到編譯器。

### 沒有垃圾回收，所以不能回傳區域變數的參考

Go 的逃逸分析會把變數搬到堆上，所以回傳區域變數的指標是安全的。Rust 不會：

```rust
// 反例：回傳區域變數的參考，無法編譯
fn dangling() -> &String {
    let text = String::from("hello");
    &text
}

fn main() {
    println!("{}", dangling());
}
```

正確做法是回傳擁有的值 `String`，讓所有權交給呼叫端。

### 沒有 nil：Option

Go 用 `nil` 與 comma-ok：

```go
package main

import "fmt"

func findAge(ages map[string]int, name string) (int, bool) {
	age, ok := ages[name]
	return age, ok
}

func main() {
	ages := map[string]int{"alice": 30}
	if age, ok := findAge(ages, "alice"); ok {
		fmt.Println("alice:", age)
	}
	if _, ok := findAge(ages, "bob"); !ok {
		fmt.Println("bob: 不存在")
	}
}
```

Rust 把「可能沒有值」放進型別 `Option<T>`，編譯器強制你處理 `None`，不會有 nil pointer dereference：

```rust
use std::collections::HashMap;

fn find_age(ages: &HashMap<String, u32>, name: &str) -> Option<u32> {
    ages.get(name).copied()
}

fn main() {
    let mut ages = HashMap::new();
    ages.insert(String::from("alice"), 30);
    if let Some(age) = find_age(&ages, "alice") {
        println!("alice: {age}");
    }
    match find_age(&ages, "bob") {
        Some(age) => println!("bob: {age}"),
        None => println!("bob: 不存在"),
    }
}
```

### 風格指南相關

- 函式參數用最通用的借用型別：`&str` 而不是 `&String`、`&[T]` 而不是 `&Vec<T>`。
- 優先使用唯一所有權與借用；`Rc` / `Arc` 只在替代方案過於複雜時使用。
- 不要用 `.clone()` 逃避借用檢查器，需要深拷貝才用。

## struct、method 與 trait
Duration: 12

### Go interface 與 Rust trait

Go 的 interface 是**隱式**實作：只要方法簽名吻合就自動滿足。

```go
package main

import "fmt"

type Shape interface {
	Area() float64
}

type Rect struct {
	Width, Height float64
}

func (r Rect) Area() float64 {
	return r.Width * r.Height
}

type Circle struct {
	Radius float64
}

func (c Circle) Area() float64 {
	return 3.14159 * c.Radius * c.Radius
}

func main() {
	shapes := []Shape{Rect{Width: 2, Height: 3}, Circle{Radius: 1}}
	total := 0.0
	for _, shape := range shapes {
		total += shape.Area()
	}
	fmt.Printf("%.2f\n", total)
}
```

Rust 的 trait 必須**明確**寫 `impl Trait for Type`：

```rust
use std::f64::consts::PI;

trait Shape {
    fn area(&self) -> f64;
}

struct Rect {
    width: f64,
    height: f64,
}

struct Circle {
    radius: f64,
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape.area()).sum()
}

fn describe(shape: &impl Shape) -> String {
    format!("面積 {:.2}", shape.area())
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Rect {
            width: 2.0,
            height: 3.0,
        }),
        Box::new(Circle { radius: 1.0 }),
    ];
    println!("總面積 {:.2}", total_area(&shapes));
    println!("{}", describe(&Circle { radius: 2.0 }));
}
```

### 兩種分派方式

| | 寫法 | 類似 Go | 特性 |
|---|------|---------|------|
| 動態分派 | `Box<dyn Shape>`、`&dyn Shape` | interface 值 | 執行期查表，可放進同一個集合 |
| 靜態分派 | `impl Shape`、`T: Shape` | 沒有直接對應（類似泛型的單態化） | 編譯期展開，沒有執行期成本 |

風格指南：在 library 中優先使用泛型（靜態分派），只有在程式碼體積很重要或需要異質集合時才用 `dyn Trait`。

### 方法接收者

| Go | Rust | 意義 |
|----|------|------|
| `func (r Rect) Area()` | `fn area(&self)` | 唯讀借用 |
| `func (r *Rect) Scale()` | `fn scale(&mut self)` | 可變借用 |
| 沒有直接對應 | `fn into_x(self)` | 取走所有權（用完就消耗） |

### 沒有繼承與 embedding

Go 的 struct embedding 提供類似繼承的方法提升。Rust 沒有，改用組合：把另一個 struct 當欄位，必要時實作 trait 轉發。

### 風格指南相關

- 只有三種情況才定義 trait：為了泛型、為了接受或回傳 `impl` / `dyn Trait`、extension trait。**不要只為了組織程式碼而定義 trait**（Go 的「先定義 interface 再實作」習慣在這裡要小心）。
- 能 `#[derive()]` 的 trait 就 derive。
- getter 不要加 `get_` 前綴：`width()` 而不是 `get_width()`，與 Go 的慣例一致。

## 錯誤處理：error 與 Result
Duration: 10

### 從 `if err != nil` 到 `?`

```go
package main

import (
	"errors"
	"fmt"
	"strconv"
)

var ErrMissing = errors.New("missing setting")

func readPort(value string) (int, error) {
	if value == "" {
		return 0, fmt.Errorf("PORT: %w", ErrMissing)
	}
	port, err := strconv.Atoi(value)
	if err != nil {
		return 0, fmt.Errorf("invalid port: %w", err)
	}
	return port, nil
}

func main() {
	for _, input := range []string{"8080", "abc", ""} {
		port, err := readPort(input)
		if errors.Is(err, ErrMissing) {
			fmt.Println("缺少設定")
			continue
		}
		if err != nil {
			fmt.Println("錯誤：", err)
			continue
		}
		fmt.Println("port =", port)
	}
}
```

Rust 的對應寫法：

```rust
use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    Missing(String),
    InvalidPort(ParseIntError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(key) => write!(formatter, "缺少設定：{key}"),
            Self::InvalidPort(error) => write!(formatter, "port 無效：{error}"),
        }
    }
}

impl Error for ConfigError {}

impl From<ParseIntError> for ConfigError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidPort(error)
    }
}

fn read_port(value: Option<&str>) -> Result<u16, ConfigError> {
    let text = value.ok_or_else(|| ConfigError::Missing(String::from("PORT")))?;
    let port = text.parse()?;
    Ok(port)
}

fn main() {
    for input in [Some("8080"), Some("abc"), None] {
        match read_port(input) {
            Ok(port) => println!("port = {port}"),
            Err(ConfigError::Missing(key)) => println!("缺少設定 {key}"),
            Err(error) => println!("錯誤：{error}"),
        }
    }
}
```

### 概念對照

| Go | Rust |
|----|------|
| `(T, error)` 兩個回傳值 | `Result<T, E>` 一個值，不是成功就是失敗 |
| `if err != nil { return err }` | `?` 運算子 |
| `fmt.Errorf("...: %w", err)` | `From` 實作加 `?` 自動轉換，或 `.map_err(...)` |
| `errors.Is` / `errors.As` | 對 enum 做 `match` |
| 哨兵錯誤變數 | enum 的變體 |
| `panic` / `recover` | `panic!`（不應用來控制流程） |

Rust 的 `?` 遇到 `Err` 會提早回傳，並自動呼叫 `From` 轉換錯誤型別，這就是為什麼上面 `text.parse()?` 能把 `ParseIntError` 轉成 `ConfigError`。

Positive
: 實務上常用 thiserror 與 anyhow 這兩個 crate 省去手寫 Display 與 From 的樣板：函式庫用 thiserror 定義具體錯誤，應用程式用 anyhow 方便地傳遞錯誤。

### 與 Go 的重要差異

- 編譯器會警告你**忽略 `Result`**（`#[must_use]`），Go 則允許你直接丟棄 `err`。
- 錯誤型別是 enum，呼叫端可以窮舉所有情況，比 `errors.Is` 的字串或變數比對更安全。

### 風格指南相關

- `Option` 用於簡單、明顯的「沒有值」；`Result` 用於呼叫端需要分辨的可恢復錯誤；`panic!` 只用於 bug 與違反不變條件。
- 用 `expect("原因")` 取代 `unwrap()`，專案已開啟 `clippy::unwrap_used`。
- 優先用 `match`、`if let`、`let ... else`，而不是先檢查再 `unwrap()`。
- 絕對不要用 `catch_unwind` 當一般錯誤處理，這不是 Go 的 `recover`。

## 並發：goroutine 與 thread
Duration: 12

### Go：goroutine 加 channel

```go
package main

import (
	"fmt"
	"sort"
	"sync"
)

func main() {
	results := make(chan int)
	var wg sync.WaitGroup
	for worker := 0; worker < 3; worker++ {
		wg.Add(1)
		go func(id int) {
			defer wg.Done()
			results <- id * 10
		}(worker)
	}
	go func() {
		wg.Wait()
		close(results)
	}()

	var collected []int
	for value := range results {
		collected = append(collected, value)
	}
	sort.Ints(collected)
	fmt.Println(collected)
}
```

### Rust：thread 加 channel

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (sender, receiver) = mpsc::channel();
    let mut handles = Vec::new();
    for worker in 0..3 {
        let sender = sender.clone();
        handles.push(thread::spawn(move || {
            sender
                .send(worker * 10)
                .expect("receiver 在 worker 結束前不會被關閉");
        }));
    }
    drop(sender);

    let mut results: Vec<i32> = receiver.iter().collect();
    results.sort_unstable();
    for handle in handles {
        handle.join().expect("worker 不應該 panic");
    }
    println!("{results:?}");
}
```

對照：

- `thread::spawn(move || ...)` 對應 `go func() {...}()`，`move` 表示把用到的變數所有權移進執行緒。
- `mpsc` 是「多生產者、單消費者」channel，`sender.clone()` 可以給多個 worker。
- `close(results)` 沒有直接對應：當**所有** `Sender` 都被 drop，`receiver.iter()` 就自然結束，所以需要 `drop(sender)` 丟掉主執行緒手上那一份。
- `wg.Wait()` 對應對每個 `JoinHandle` 呼叫 `join()`。

### 共享狀態：`Arc<Mutex<T>>`

Go 的 `sync.Mutex` 與資料是分開的，你得自己記得鎖。Rust 的 `Mutex<T>` **把資料包在鎖裡面**，不拿鎖就拿不到資料：

```rust
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::new();
    for _ in 0..4 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            let mut guard = counter.lock().expect("mutex 不應該被汙染");
            *guard += 1;
        }));
    }
    for handle in handles {
        handle.join().expect("worker 不應該 panic");
    }
    println!("計數 = {}", *counter.lock().expect("mutex 不應該被汙染"));
}
```

- `Arc` 是執行緒安全的參考計數指標，讓多個執行緒共同擁有同一份資料。
- `lock()` 回傳的 guard 離開作用域就自動解鎖，不需要 `defer mu.Unlock()`。
- `lock()` 回傳 `Result`：如果持鎖的執行緒 panic，鎖會被標記為「汙染（poisoned）」。

### 編譯器擋下資料競爭

在 Rust，如果你把非執行緒安全的型別（例如 `Rc`）送進另一個執行緒，**編譯就會失敗**。這由 `Send` 與 `Sync` 這兩個 trait 在編譯期檢查，不需要 `-race` 去賭執行期剛好踩到。

### async 與 tokio（下一章有完整範例）

Rust 的標準函式庫沒有內建 goroutine 風格的 runtime。需要大量輕量並發（例如網路服務）時，使用 `async` / `await`，並搭配 `tokio` 這類 crate 提供的 runtime：

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

| Go | tokio |
|----|-------|
| `go f()` | `tokio::spawn(f())` |
| `select` | `tokio::select!` |
| `context.Context` 取消 | `CancellationToken` 或 `select!` |
| channel | `tokio::sync::mpsc` |

Negative
: async Rust 的學習曲線比 goroutine 陡峭。先學會同步與 thread，再進入 async，會比較順。

### 風格指南相關

- 永遠不要去取用已汙染（poisoned）的 mutex 內的資料，所以用 `expect` 讓程式明確失敗。
- 優先使用唯一所有權與借用；`Arc` 只在真的需要跨執行緒共享時使用，並用 `Arc::clone(&x)` 表示意圖。

## 測試、文件與專案結構
Duration: 8

### 表格驅動測試

Go 的經典寫法：

```go
package main

import "testing"

func isEven(number int) bool {
	return number%2 == 0
}

func TestIsEven(t *testing.T) {
	cases := []struct {
		input    int
		expected bool
	}{
		{0, true},
		{1, false},
		{4, true},
		{-3, false},
	}
	for _, tc := range cases {
		if got := isEven(tc.input); got != tc.expected {
			t.Errorf("isEven(%d) = %v, want %v", tc.input, got, tc.expected)
		}
	}
}
```

Rust 把單元測試寫在同一個檔案的 `tests` 模組：

```rust
fn is_even(number: i32) -> bool {
    number % 2 == 0
}

fn main() {
    println!("4 是偶數：{}", is_even(4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_even_table() {
        let cases = [(0, true), (1, false), (4, true), (-3, false)];
        for (input, expected) in cases {
            assert_eq!(is_even(input), expected, "is_even({input})");
        }
    }
}
```

### 工具對照

| Go | Rust |
|----|------|
| `foo_test.go`（同目錄） | 同檔 `#[cfg(test)] mod tests`，或 `tests/` 目錄做整合測試 |
| `go test ./...` | `cargo test --workspace` |
| `Example` 函式 | 文件註解中的程式碼區塊，`cargo test` 會執行 |
| `go test -bench` | `cargo bench`（stable 需用 criterion 等 crate） |
| `go test -race` | 不需要，編譯期已檢查 |
| `go doc` | `cargo doc --open`，註解以 `///` 撰寫 |

### 專案結構對照

```text
Go                              Rust
├── go.mod                      ├── Cargo.toml
├── cmd/app/main.go             ├── src/main.rs  或  src/bin/app.rs
├── internal/store/             ├── src/store.rs  （預設 private）
├── pkg/greeter/                ├── crates/greeter/  （workspace 成員）
└── greeter_test.go             └── tests/greeter.rs
```

### 可見性

| Go | Rust |
|----|------|
| 首字母大寫＝匯出 | `pub` |
| 小寫＝套件內私有 | 預設私有（模組內） |
| `internal/` 目錄 | `pub(crate)`：crate 內可見 |

Rust 的欄位也可以個別決定是否公開，粒度比 Go 的整個 package 更細。

### 風格指南相關

- 檔名與 leaf 模組以主要型別命名：`Foo` 放在 `foo.rs`。
- 非 leaf 模組只放 `mod` 宣告與 re-export。
- 公開 API 都要有 `///` 文件，註明 panic 與錯誤條件，模組與 crate 用 `//!`。
- `assert!` 與 `assert_eq!` 要附上說明訊息。
- 不使用 `use x::*`，只有測試模組可以用 `use super::*`。

## async 與 tokio：goroutine 的另一種對應
Duration: 15

這一章用一個**可實際執行的範例專案** `examples/async-tokio/` 來教學：每個 Go 並發模式都有對應的 Rust 寫法，專案由 CI 以 `cargo fmt`、`clippy -D warnings` 與 `cargo test` 檢查。

### 三個和 goroutine 最大的差異

1. **沒有內建 runtime**：`async` 只是語法，要有執行 future 的 runtime 才會動。絕大多數專案使用 tokio。
2. **future 是惰性的**：呼叫 `async fn` 只會得到一個 future，不 `.await` 也不 `spawn` 就什麼都不會發生。Go 的 `go f()` 一呼叫就開始跑。
3. **編譯器檢查資料競爭**：丟進 `tokio::spawn` 的 future 必須是 `Send`，不安全的共享會在編譯期就被擋下。

Negative
: async Rust 的學習曲線比 goroutine 陡峭。如果你只是要同時做幾件事，先用上一章的 thread 與 mpsc 就夠了，真的需要大量輕量並發（網路服務）才引入 async。

### 專案設定

`examples/async-tokio/Cargo.toml`：

```toml
[dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread", "sync", "time"] }
tokio-util = "0.7"

[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
```

- `macros`：提供 `#[tokio::main]`、`#[tokio::test]` 與 `select!`。
- `rt-multi-thread`：多執行緒的排程器，類似 Go 的 GOMAXPROCS 多核排程。
- `sync`：`mpsc`、`oneshot` 等同步原語。
- `time`：`sleep`、`timeout`。
- `test-util`：測試用的**虛擬時間**，只放在 `dev-dependencies`。

Positive
: 初學時可以直接寫 features = ["full"]，但正式專案建議只開需要的功能，編譯更快、依賴也更少。

### 對照總表

| Go | tokio |
|----|-------|
| `go f()` | `tokio::spawn(f())` |
| `sync.WaitGroup` | `JoinSet`，或對 `JoinHandle` 做 `.await` |
| `chan T` | `tokio::sync::mpsc` |
| 只傳一次結果的 channel | `tokio::sync::oneshot` |
| 廣播、最新值 | `tokio::sync::broadcast`、`watch` |
| `select` | `tokio::select!` |
| `time.After` | `tokio::time::sleep`、`tokio::time::timeout` |
| `context.WithCancel` | `tokio_util::sync::CancellationToken` |
| `errgroup` | `JoinSet`、`tokio::try_join!` |
| `sync.Once` | `std::sync::OnceLock` |

### 模式一：spawn 與等待全部完成

Go 用 goroutine 加 WaitGroup（上一章已示範）。tokio 用 `JoinSet`，它同時負責「啟動」與「收集結果」：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 的 goroutine 加 WaitGroup：開出多個 task 並等全部完成。
async fn run_workers() -> Vec<u32> {
    let mut tasks = JoinSet::new();
    for worker in 0..3_u32 {
        tasks.spawn(async move { worker * 10 });
    }
    let mut results = Vec::new();
    while let Some(outcome) = tasks.join_next().await {
        results.push(outcome.expect("worker 不應該 panic"));
    }
    results.sort_unstable();
    results
}
```

- `tasks.spawn(...)` 開出一個 task。
- `join_next().await` 每次回傳下一個完成的 task 結果，沒有 task 時回傳 `None`。
- 結果是 `Result`，task 如果 panic 就是 `Err`，所以用 `expect` 說明為什麼不應該發生。
- `JoinSet` 被 drop 時會**取消所有還沒完成的 task**，不會留下孤兒。

### 模式二：channel

Go：

```go
package main

import "fmt"

func main() {
	values := make(chan int, 8)
	go func() {
		defer close(values)
		for value := 1; value <= 3; value++ {
			values <- value
		}
	}()
	total := 0
	for value := range values {
		total += value
	}
	fmt.Println(total)
}
```

Rust：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 的 `chan int`：生產者送出 1 到 3，消費者加總。
async fn sum_from_channel() -> u32 {
    let (sender, mut receiver) = mpsc::channel(8);
    tokio::spawn(async move {
        for value in 1..=3_u32 {
            sender.send(value).await.expect("receiver 仍存在");
        }
    });
    let mut total = 0;
    while let Some(value) = receiver.recv().await {
        total += value;
    }
    total
}
```

對照：

- `mpsc::channel(8)` 的 8 是緩衝容量，和 `make(chan int, 8)` 相同；容量滿了 `send().await` 會等待。
- Go 要自己 `close(values)`；Rust 在**所有 `Sender` 都被 drop** 時 channel 自動關閉，`recv()` 回傳 `None`。這裡 `sender` 被移進 task，task 結束就 drop。
- `while let Some(value) = receiver.recv().await` 對應 `for value := range values`。

### 模式三：oneshot

Go 常用容量 1 的 channel 傳一次結果：

```go
package main

import "fmt"

func main() {
	result := make(chan int, 1)
	go func() {
		result <- 6 * 7
	}()
	fmt.Println(<-result)
}
```

tokio 有專用的 `oneshot`，型別上就表達了「只會送一次」：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 中「只傳一次結果」的 channel。
async fn compute_in_background() -> u32 {
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        sender.send(6 * 7).expect("receiver 仍在等待");
    });
    receiver.await.expect("sender 不應該在送出前被丟棄")
}
```

`oneshot::Sender::send` 會消耗自己（所有權移轉），所以**不可能**送第二次，這是 Go 的 channel 做不到的編譯期保證。

### 模式四：select 與逾時

Go：

```go
package main

import (
	"fmt"
	"time"
)

func fetchWithTimeout(delay, limit time.Duration) (string, bool) {
	done := make(chan string, 1)
	go func() {
		time.Sleep(delay)
		done <- "完成"
	}()
	select {
	case result := <-done:
		return result, true
	case <-time.After(limit):
		return "", false
	}
}

func main() {
	fmt.Println(fetchWithTimeout(50*time.Millisecond, 500*time.Millisecond))
	fmt.Println(fetchWithTimeout(500*time.Millisecond, 50*time.Millisecond))
}
```

Rust：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 的 `select` 加 `time.After`：工作超過時限就放棄。
async fn fetch_with_timeout(delay: Duration, limit: Duration) -> Option<&'static str> {
    let work = async {
        time::sleep(delay).await;
        "完成"
    };
    tokio::select! {
        result = work => Some(result),
        () = time::sleep(limit) => None,
    }
}
```

重要差異：**`select!` 選中一個分支後，其他分支的 future 會被 drop，也就是被取消**。上面逾時分支勝出時，`work` 會在下一個 `.await` 的位置停止。Go 的做法則是 goroutine 仍在背景跑（甚至洩漏），要自己用 context 通知它停止。

Positive
: 單純的逾時有更簡短的寫法：time::timeout(limit, work).await 會回傳 Result，逾時時是 Err。select! 則用在要同時等多個不同來源的情況。

### 模式五：取消（context）

Go 用 `context.Context` 傳遞取消訊號：

```go
package main

import (
	"context"
	"fmt"
	"time"
)

func tickUntilCancelled(ctx context.Context, ticks chan<- int) {
	count := 0
	for {
		select {
		case <-ctx.Done():
			return
		case <-time.After(10 * time.Millisecond):
			count++
			ticks <- count
		}
	}
}

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	ticks := make(chan int, 64)
	done := make(chan struct{})
	go func() {
		tickUntilCancelled(ctx, ticks)
		close(done)
	}()
	time.Sleep(35 * time.Millisecond)
	cancel()
	<-done
	close(ticks)
	received := 0
	for range ticks {
		received++
	}
	fmt.Println(received)
}
```

Rust 用 `tokio-util` 的 `CancellationToken`：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 的 `context.WithCancel`：每 10 毫秒送出一次計數，直到被取消。
async fn tick_until_cancelled(token: CancellationToken, ticks: mpsc::Sender<u32>) {
    let mut count = 0;
    loop {
        tokio::select! {
            () = token.cancelled() => break,
            () = time::sleep(Duration::from_millis(10)) => {
                count += 1;
                if ticks.send(count).await.is_err() {
                    break;
                }
            }
        }
    }
}
```

呼叫端負責啟動、等待後取消，並收集結果：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 啟動計數 task，等待一段時間後取消，回傳收到的計數次數。
async fn count_ticks(run_for: Duration) -> usize {
    let token = CancellationToken::new();
    let (sender, mut receiver) = mpsc::channel(64);
    let handle = tokio::spawn(tick_until_cancelled(token.clone(), sender));
    time::sleep(run_for).await;
    token.cancel();
    handle.await.expect("計數 task 不應該 panic");
    let mut received = 0;
    while receiver.recv().await.is_some() {
        received += 1;
    }
    received
}
```

- `token.clone()` 是便宜的，可以分給多個 task，任何一份呼叫 `cancel()` 都會讓所有 `cancelled()` 完成。
- 和 Go 的 `context` 不同，token **不是每個函式的第一個參數的慣例**，也沒有內建逾時與 value；逾時用 `timeout`，傳值用一般參數。

### 模式六：只初始化一次（sync.Once）

Go：

```go
package main

import (
	"fmt"
	"sync"
)

var (
	appName     string
	appNameOnce sync.Once
)

func getAppName() string {
	appNameOnce.Do(func() {
		appName = "async-tokio-demo"
	})
	return appName
}

func main() {
	fmt.Println(getAppName())
}
```

Rust：

```rust
// excerpt: examples/async-tokio/src/main.rs
/// 對應 Go 的 `sync.Once`：第一次呼叫時才初始化，之後重複使用。
fn app_name() -> &'static str {
    static APP_NAME: OnceLock<String> = OnceLock::new();
    APP_NAME.get_or_init(|| String::from("async-tokio-demo"))
}
```

`OnceLock<T>` 把「值」與「只初始化一次」綁在一起，不需要像 Go 那樣另外放一個變數。`OnceLock` 自 Rust 1.70 穩定；`LazyLock`（宣告時就給初始化函式）自 1.80 穩定。

### 在 main 串起來

```rust
// excerpt: examples/async-tokio/src/main.rs
#[tokio::main]
async fn main() {
    println!("應用程式：{}", app_name());
    println!("workers：{:?}", run_workers().await);
    println!("channel 加總：{}", sum_from_channel().await);
    println!("oneshot：{}", compute_in_background().await);
    let fast = fetch_with_timeout(Duration::from_millis(50), Duration::from_millis(500)).await;
    let slow = fetch_with_timeout(Duration::from_millis(500), Duration::from_millis(50)).await;
    println!("快的工作：{fast:?}，慢的工作：{slow:?}");
    println!(
        "取消前的計數：{}",
        count_ticks(Duration::from_millis(35)).await
    );
}
```

`#[tokio::main]` 會建立 runtime 並把 `async fn main` 跑到結束，等同於你在同步的 `main` 裡手動建立 runtime 再 `block_on`。

### 測試非同步程式碼：虛擬時間

```rust
// excerpt: examples/async-tokio/src/main.rs
    #[tokio::test(start_paused = true)]
    async fn timeout_only_when_too_slow() {
        let one_second = Duration::from_secs(1);
        let fast = fetch_with_timeout(Duration::from_millis(100), one_second).await;
        let slow = fetch_with_timeout(Duration::from_secs(2), one_second).await;
        assert_eq!(fast, Some("完成"), "快的工作應該在時限內完成");
        assert_eq!(slow, None, "慢的工作應該逾時");
    }
```

`start_paused = true` 讓測試使用**虛擬時間**：所有 `sleep` 與 `timeout` 瞬間完成，但先後順序仍然正確，測試既快又不會因機器忙碌而不穩定。範例的逾時與取消測試都靠這個，沒有任何一個真的等待。Go 要做到這點，需要自己注入時鐘，或使用較新版本才有的 `testing/synctest`。

### 常見陷阱

| 陷阱 | 說明 |
|------|------|
| 在 async 裡做阻塞呼叫 | 例如 `std::thread::sleep` 或同步檔案 I/O，會卡住整條 worker 執行緒。改用 `tokio::time::sleep`，或用 `tokio::task::spawn_blocking` |
| 持有 `std::sync::Mutex` 跨過 `.await` | 容易造成死結，且 guard 不是 `Send`，編譯器會擋。改用 `tokio::sync::Mutex`，或縮小鎖的範圍 |
| 忘了 `.await` | future 沒被執行；編譯器會警告「unused future」，不要忽略 |
| 以為 drop `JoinHandle` 會取消 task | 不會，task 會繼續在背景跑（detach）。要能取消就用 `JoinSet` 或 `abort()` |
| `select!` 的分支被取消 | 被捨棄的 future 可能做到一半，必須是「取消安全」的 |

### 風格指南相關

- Google Rust Style Guide 對 async 沒有一般性的禁止；它提到「不使用 `async` / `.await`」是**特定專案（Chrome）**的規則，不是通用建議。
- 風格指南的**「不要碰已被汙染的 mutex」**原則，在 async 同樣適用：對 `lock()` 的結果用 `expect` 說明理由。
- 依賴要審慎：`tokio` 加上 `tokio-util` 會帶入一棵依賴樹，這個範例專案也納入了 `cargo audit` 與 `cargo deny` 的掃描。

### 執行範例

```console
cd examples/async-tokio
cargo run
cargo test
cargo clippy --all-targets -- -D warnings
```

執行結果：

```text
應用程式：async-tokio-demo
workers：[0, 10, 20]
channel 加總：6
oneshot：42
快的工作：Some("完成")，慢的工作：None
取消前的計數：3
```

## 建置、跨平台編譯與容器化
Duration: 12

### 跨平台編譯：GOOS / GOARCH 與 target

Go 的跨平台編譯非常輕鬆，因為工具鏈自帶各平台的編譯器與連結器：

```console
GOOS=linux GOARCH=arm64 go build
```

Rust 也能跨平台編譯，但需要自己準備更多東西：

| | Go | Rust |
|---|----|------|
| 指定目標 | `GOOS` 與 `GOARCH` | `--target <三元組>`，例如 `aarch64-unknown-linux-gnu` |
| 標準函式庫 | 內建所有平台 | 先 `rustup target add <三元組>` |
| 連結器 | 自帶 | **使用系統的連結器**，跨平台時要另外安裝並設定 |
| 靜態執行檔 | 預設靜態（`CGO_ENABLED=0`） | 預設動態連結 glibc；用 `musl` 目標可靜態連結 |
| 列出支援平台 | `go tool dist list` | `rustc --print target-list` |

三元組的格式是「架構-廠商-系統」，例如 `x86_64-unknown-linux-musl`、`aarch64-apple-darwin`、`x86_64-pc-windows-msvc`。

### 編出靜態執行檔

```console
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

產物在 `target/x86_64-unknown-linux-musl/release/`。這等同於 Go 的 `CGO_ENABLED=0 go build`，適合放進最小的容器映像。

Positive
: 本專案用這個方式在 Docker 內建置，並以 file 指令確認產物是 static-pie linked，也就是完全靜態的執行檔。

### 真正的跨平台：不要硬在一台機器上編

連結器是跨平台編譯最麻煩的地方，常見的處理方式：

| 方式 | 適用情境 |
|------|----------|
| **`cargo-zigbuild`** | 用 zig 當連結器，在 Linux 上編各種 Linux 架構與指定 glibc 版本，不用裝 gcc 交叉工具鏈 |
| **`cross`** | 在預先配好工具鏈的 Docker 容器內編譯 |
| **CI 矩陣（推薦）** | 在 GitHub Actions 讓每個平台各用自己的 runner 原生編譯 |

Negative
: 從 Linux 編 macOS 或 Windows（MSVC）執行檔需要 SDK 或額外工具（例如 cargo-xwin），設定成本高。直接用 macos-latest 與 windows-latest 的 runner 編，是最省事的做法。

### GitHub Actions 多平台矩陣

`.github/workflows/build-binaries.yml`（節錄）：

```yaml
jobs:
  binaries:
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: ubuntu-24.04-arm
            target: aarch64-unknown-linux-musl
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4
      - run: rustup target add ${{ matrix.target }}
      - if: contains(matrix.target, 'musl')
        run: sudo apt-get update && sudo apt-get install -y musl-tools
      - run: cargo build --release --locked --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: rust_practice-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/rust_practice${{ runner.os == 'Windows' && '.exe' || '' }}
```

`--locked` 要求建置必須完全依照 `Cargo.lock`，如果 lock 與 `Cargo.toml` 不一致就失敗，確保 CI 的結果可重現。

### Dockerfile：多階段建置

Go 與 Rust 的容器化想法相同：編譯器只留在建置階段，最終映像只含執行檔。

`Dockerfile`（動態連結 glibc，執行階段用 distroless）：

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:1.78 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/rust_practice /rust_practice

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /rust_practice /rust_practice
ENTRYPOINT ["/rust_practice"]
```

`Dockerfile.musl`（靜態連結，執行階段是 scratch）：

```dockerfile
# syntax=docker/dockerfile:1
FROM rust:1.78 AS builder
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked --target x86_64-unknown-linux-musl \
    && cp target/x86_64-unknown-linux-musl/release/rust_practice /rust_practice

FROM scratch
COPY --from=builder /rust_practice /rust_practice
USER 65532:65532
ENTRYPOINT ["/rust_practice"]
```

### 實測結果

| 映像 | 基底 | 大小 |
|------|------|------|
| `rust-practice:glibc` | `distroless/cc-debian12` | 24.1 MB |
| `rust-practice:musl` | `scratch` | 3.98 MB |

兩個映像都能正常執行並輸出程式內容。

### 與 Go 容器化的差異

- **glibc 與 musl 的選擇**：Go 預設靜態，所以幾乎都能放進 `scratch`。Rust 預設動態連結，要用 `scratch` 必須改編 musl 目標，或改用 `distroless/cc` 這類帶 glibc 的精簡基底。
- **依賴快取**：Go 的 module cache 容易掛載。Rust 沒有快取時，只要改一行原始碼就會重新編譯所有依賴。上面用 BuildKit 的 `--mount=type=cache` 保留 registry 與 `target/`；依賴很多的專案也常用 `cargo-chef` 把「編依賴」與「編自己的程式」拆成不同的 layer。
- **musl 的注意事項**：musl 的記憶體配置器效能通常比 glibc 慢，對配置很頻繁的程式，要實測；部分依賴 C 函式庫的 crate 也可能不好在 musl 上編譯。
- **`.dockerignore`**：一定要排除 `target/`，否則會把數 GB 的編譯產物送進 build context。
- **非 root 執行**：distroless 提供 `:nonroot` 標籤；`scratch` 用 `USER 65532:65532`。

### 風格指南與安全相關

- 建置時加 `--locked`，與 `Cargo.lock` 不一致就失敗，防止依賴被悄悄更新。
- Dependabot 的 `docker` 生態可以追蹤 Dockerfile 的基底映像版本。
- `rust:1.78` 這類固定版本的基底映像能讓建置可重現；升級由 Dependabot 提醒。

## CI 與供應鏈安全：golangci-lint 與 govulncheck
Duration: 12

### 工具對照

| Go | Rust | 做什麼 |
|----|------|--------|
| `go vet`、golangci-lint | `cargo clippy` | 靜態分析與慣用寫法檢查 |
| `gofmt -l` | `cargo fmt --check` | 格式檢查 |
| `govulncheck` | `cargo audit` | 以 RustSec 資料庫檢查已知漏洞 |
| go-licenses 等工具 | `cargo deny` | 授權、禁用套件、依賴來源、漏洞 |
| `go mod tidy` 清除多餘依賴 | `cargo machete` | 找出沒用到的依賴 |
| `gorelease` | `cargo semver-checks` | 發布函式庫前檢查是否破壞相容 |
| Dependabot（gomod） | Dependabot（cargo） | 自動開依賴升級 PR |

`cargo audit`、`cargo deny`、`cargo machete` 都是需要另外安裝的 Cargo 子指令，不是 Rust 工具鏈內建。

### govulncheck 與 cargo audit 的差別

- **govulncheck** 會分析呼叫圖，只回報你的程式碼**實際呼叫到**有漏洞函式的情況。
- **cargo audit** 只比對 `Cargo.lock` 裡的版本與 RustSec 資料庫，只要依賴樹裡有受影響的版本就會回報，不管你有沒有真的用到。

所以 cargo audit 的誤報會比較多，遇到時要自行判斷，確認不受影響時才用 `ignore` 並寫明理由。

Negative
: 沒有依賴的專案，cargo audit 與 cargo deny 實際上沒有東西可掃。把工具先設好，等加入依賴後才會開始發揮作用。

### 第一層：clippy 是 golangci-lint

本課程專案已經在 `Cargo.toml` 設定了 lint，並在 CI 以 `-D warnings` 把警告當成錯誤：

```console
cargo clippy --all-targets -- -D warnings
```

### 第二層：deny.toml

`cargo deny` 的設定放在專案根目錄的 `deny.toml`：

```toml
[advisories]
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-3.0",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

| 區塊 | 作用 |
|------|------|
| `[advisories]` | 已知漏洞與被棄用的套件 |
| `[licenses]` | 授權白名單，不在名單內的授權會失敗 |
| `[bans]` | 禁用特定套件；`wildcards = "deny"` 禁止 `"*"` 版本需求 |
| `[sources]` | 只允許 crates.io，擋下未知的 registry 與 git 依賴 |

在本機執行：

```console
cargo install cargo-audit cargo-deny cargo-machete
cargo audit
cargo deny check
cargo machete
```

Positive
: cargo-deny 的設定格式隨版本演進，欄位若有變動，請以官方文件為準：embarkstudios.github.io/cargo-deny。

### 第三層：GitHub Actions 定期掃描

`.github/workflows/security.yml`：

```yaml
name: Security

on:
  push:
    branches: [main, master]
    paths: ['Cargo.toml', 'Cargo.lock', 'deny.toml']
  pull_request:
    paths: ['Cargo.toml', 'Cargo.lock', 'deny.toml']
  schedule:
    - cron: '0 2 * * 1'
  workflow_dispatch:

permissions:
  contents: read

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: taiki-e/install-action@v2
        with:
          tool: cargo-audit,cargo-deny,cargo-machete
      - run: cargo audit
      - run: cargo deny check
      - run: cargo machete
```

重點：

- **`schedule`**：每週掃一次。新漏洞會在你沒改任何程式碼的時候公開，所以光靠「改依賴時才檢查」不夠。
- **`paths`**：只有依賴相關檔案改變時才在 push 與 PR 觸發，避免每次提交都跑。
- **`taiki-e/install-action`**：直接下載預編譯的工具，比 `cargo install` 從原始碼編譯快很多。
- **`permissions: contents: read`**：最小權限原則。

### 第四層：Dependabot

`.github/dependabot.yml`：

```yaml
version: 2
updates:
  - package-ecosystem: cargo
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
```

這會每週檢查 Cargo 依賴與 workflow 裡使用的 Action 版本，有更新就自動開 PR；CI 通過後再合併，可以讓依賴持續保持最新。

### 其他可考慮的工具

- **`cargo semver-checks`**：發布函式庫前使用，檢查是否不小心破壞公開 API 的相容性。
- **CodeQL**：GitHub 內建的程式碼掃描，可在 repo 的 Security 設定開啟。
- **`cargo geiger`**：統計依賴樹中 `unsafe` 的使用量。

### 風格指南相關

- Cargo.toml 的 `[lints.rust] unsafe_code = "forbid"` 已經禁止自己的程式碼使用 `unsafe`；風格指南要求 `unsafe` 必須包在安全的抽象裡並附 `SAFETY:` 註解。
- 依賴要有明確版本，`cargo deny` 的 `wildcards = "deny"` 可以自動擋下 `"*"`。
- 每增加一個依賴就多一份維護與安全責任，加入前先確認真的需要。

Negative
: 本章的工具與工作流程設定，是由 GitHub Actions 實際執行時才會驗證，本課程建置時的範例檢查只涵蓋 Go 與 Rust 程式碼，不包含這些設定檔。

## 速查表與下一步
Duration: 5

### Go → Rust 速查

| 想做的事 | Go | Rust |
|----------|----|------|
| 套件管理 | `go mod` | `cargo` |
| 格式化 | `gofmt` | `cargo fmt` |
| 靜態檢查 | `go vet` | `cargo clippy` |
| 漏洞掃描 | `govulncheck` | `cargo audit`、`cargo deny` |
| 跨平台編譯 | `GOOS` / `GOARCH` | `--target` 加 `rustup target add` |
| 靜態執行檔 | `CGO_ENABLED=0` | `x86_64-unknown-linux-musl` |
| 空值 | `nil` | `Option<T>` |
| 錯誤 | `(T, error)` | `Result<T, E>` 與 `?` |
| 抽象 | interface（隱式） | trait（明確） |
| 動態分派 | interface 值 | `Box<dyn Trait>` |
| 並發 | `go` 與 channel | `thread::spawn` 與 `mpsc`，async 用 tokio |
| 共享狀態 | `sync.Mutex` | `Arc<Mutex<T>>` |
| select、逾時、取消 | `select`、`time.After`、`context` | `tokio::select!`、`timeout`、`CancellationToken` |
| 只初始化一次 | `sync.Once` | `OnceLock` |
| 零值 | 自動 | `Default` |
| 記憶體 | GC | 所有權與借用 |
| 測試 | `_test.go` | `#[cfg(test)]` |

### Go 開發者最常踩的坑

1. **以為賦值是共享**：Rust 的賦值是移動，要共用就借用或 `Arc`。
2. **想用 `nil`**：改用 `Option`，並習慣用 `match` 與 `if let`。
3. **到處 `.clone()` 逃避編譯器**：先想能不能借用。
4. **先定義 interface 再實作**：Rust 只在需要泛型或 `dyn` 時才定義 trait。
5. **忽略錯誤**：Rust 不讓你悄悄丟掉 `Result`。
6. **期待 goroutine 的輕量**：Rust 的 thread 是系統執行緒，大量並發請看 async。

### 提交前的一鍵檢查

```console
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

這就是 Rust 版的 `gofmt -l`、`go vet ./...`、`go test ./...`。

### 延伸閱讀

- [Google Rust Style Guide](https://google.github.io/styleguide/rustguide.html)
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Comprehensive Rust（Google）](https://google.github.io/comprehensive-rust/)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)

Positive
: 完成！你已經把 Go 的核心概念對應到 Rust，接下來最好的練習，就是把一個你寫過的 Go 小工具用 Rust 重寫一次。
