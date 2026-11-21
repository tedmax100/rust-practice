author: Nathan
summary: 揭秘 Rust 記憶體安全的真實代價、編譯器動態邊界檢查與自動向量化瓶頸，以及不碰 unsafe 榨乾 CPU 的五大實戰技巧
id: rust-performance-safety
categories: rust,performance,advanced
environments: Web
status: Published

# 安全的代價：揭秘 Rust 效能（一）不碰 unsafe 也能榨乾 CPU

## 課程簡介與核心張力
Duration: 6

### 歡迎來到 Rust 效能底層世界

長期以來，系統級與底層開發者之間一直存在激烈的爭論：**Rust 為了追求極致的記憶體安全，到底付出了多少效能代價？**

許多剛接觸 Rust 的工程師常有一種迷思，認為要追求極限效能就必須大量依賴 `unsafe` 區塊來繞過編譯器檢查。然而事實恰恰相反：**在死守 100% 記憶體安全底線的前提下，完全不碰任何 `unsafe` 程式碼，我們依然能將 CPU 的剩餘運算效能全部榨乾。**

```
      +-------------------------------------------------------+
      |               Rust 語言的核心設計張力                  |
      +-------------------------------------------------------+
      |  第一優先級：記憶體安全 (Memory Safety First)            |
      |  第二優先級：極致速度 (Speed Closely Second)            |
      +-------------------------------------------------------+
                                 │
                 ┌───────────────┴───────────────┐
                 ▼                               ▼
       靜態保證 (Zero-Cost)             動態硬性支出 (Overhead)
    • 所有權與生命週期借用檢查         • 變數與未初始化記憶體強制歸零
    • 編譯期單態化 (Monomorphization)  • 陣列與切片動態邊界檢查 (Bounds Check)
    • 獨占借用帶來的別名分析紅利       • 阻礙迴圈自動向量化 (SIMD)
```

### 安全第一，速度緊隨其後

Rust 核心團隊前成員 Steve Klabnik 曾說過一句極具代表性的話：

> 「Rust 絕對把記憶體安全放在第一位，速度緊隨其後排在第二。」

這句話一語道破了 Rust 的核心張力——它要在「絕對安全」與「極致速度」之間走鋼絲：

- **零成本抽象（Zero-cost Abstractions）**：高階的語法糖、迭代器、閉包、RAII 在編譯為機器碼後，不會比手寫的低階 C 語言多出執行期包袱。
- **安全防線的硬性支出**：為了杜絕記憶體未定義行為（Undefined Behavior, UB），編譯器必須在底層插入強制檢查。最顯著的兩大硬性支出就是**記憶體強制初始化**與**無處不在的動態邊界檢查（Dynamic Bounds Checking）**。

Positive
: 本教學所有技巧均遵守 Google Rust Style Guide，且全程在安全的 Safe Rust 範疇內運作。專案啟用 unsafe_code = "forbid"，絕不使用 unsafe 規避安全檢查。

## 安全的真實代價：這筆支出到底值不值？
Duration: 6

### 緩衝區溢位：黑客最順手的毀滅武器

要理解邊界檢查的價值，必須先看清「緩衝區溢位（Buffer Overflow）」的本質：

這就像拿一個水壺往玻璃杯裡倒水，杯子的容量是固定的；水一旦倒多了，液體就會溢出杯緣流得到處都是。對應到電腦系統中，就是程式碼將資料寫入了超出目標容器記憶體邊界的地方，覆蓋掉了相鄰的變數、暫存器備份或函式返回位址（Return Address）。

在系統程式領域，這一直是破壞力最強、攻擊者最順手的攻擊手段之一。

```
記憶體配置視圖：
+-----------------------+-----------------------+-----------------------+
|  合法緩衝區 Buffer    |  相鄰重要業務變數     |  函式返回位址 (RIP)   |
|  [ 0 ][ 1 ][ 2 ][ 3 ] |  is_admin = false     |  0x7fff_cafe_babe     |
+-----------------------+-----------------------+-----------------------+
  ▲                   ▲
  │   正常合法存取    │   緩衝區溢位寫入 (越界存取)
  └───────────────────┴─────────────────────────────────────────►
                        直接覆蓋相鄰資料，竊取 root 權限或跳轉惡意 Shellcode
```

### 殘酷的現實數據：70% 以上的系統漏洞

動態邊界檢查絕非編譯器沒事找事，世界各大科技巨頭數十年的資安數據已經給出答案：

| 機構與統計來源 | 漏洞比例 | 核心根因 |
|---|---|---|
| 微軟安全中心（MSRC）歷史統計 | **約 70%** | 記憶體安全問題（記憶體溢位與 Use-After-Free） |
| Google Chromium 專案長年追蹤 | **約 70%** | C/C++ 撰寫元件中的越界存取與指針懸垂 |
| 國際權威機構 2024 年漏洞分析 | **超過 80%** | 重大記憶體安全 CVE 均由溢位與未受控邊界直接引起 |

正是為了解決這 70% 到 80% 的致命漏洞，Rust 才在所有陣列與切片索引存取時，預設注入邊界檢查。這道防線是現代安全系統程式設計的硬性底線。

Negative
: 關閉邊界檢查雖然能在極端微基準測試（Microbenchmarks）中換取些許數字增長，但等於主動拆毀 Rust 最強大的安全護城河，將整個系統重新暴露給緩衝區溢位攻擊。

## 邊界檢查如何廢掉編譯器優化
Duration: 8

### CPU 的實打實物理損耗

在 CPU 微架構層面，每一次看似微不足道的邊界檢查，都有實打實的物理開銷：

1. **算術邏輯單元（ALU）週期消耗**：每一次索引存取 `slice[i]`，都必須執行 `cmp index, len` 與條件分支跳轉指令 `jge panic`。
2. **暫存器壓力（Register Pressure）**：切片的長度（`len`）與當前索引必須常駐在有限的通用暫存器中，排擠了其他熱點資料的暫存機會，導致更多溢出到堆疊（Register Spilling）。
3. **指令快取（I-Cache）膨脹**：大量的分支比對與 panic 處理跳轉路徑，把 CPU 一級指令快取塞得滿滿當當，降低快取命中率。

### 致命打擊：自動向量化（Auto-Vectorization）的毀滅

最嚴重的效能損害發生在百萬次的高頻迴圈（Hot Loops）內部：

現代 CPU 都具備 SIMD（單指令多資料流）運算單元（如 AVX2、AVX-512、NEON），就像集裝箱碼頭一樣，可以一條指令平行處理 4 到 16 個數字。LLVM 的迴圈向量化器（Loop Vectorizer）會嘗試自動將純量迴圈升級為向量化指令。

然而，**迴圈內部的邊界檢查會徹底打斷這一優化過程**。因為編譯器無法證明某一次迭代是否會因為越界而觸發 `panic`，為了維持精確的異常語義與副作用順序，編譯器只能保守地放棄向量化，退化為逐個元素的純量指令。這無異於給在高速公路上狂飆的跑車硬生生拉上手剎！

```
理想的 SIMD 向量化處理（一次處理 4 個 64-bit 元素）：
Reg YMM0: [ a[0] | a[1] | a[2] | a[3] ]  * 2  ==>  [ r[0] | r[1] | r[2] | r[3] ]

被邊界檢查中斷後的純量退化：
Iteration 0: 檢查 index 0 < len? ──► 計算 a[0] * 2 ──► 寫入
Iteration 1: 檢查 index 1 < len? ──► 計算 a[1] * 2 ──► 寫入
Iteration 2: 檢查 index 2 < len? ──► 計算 a[2] * 2 ──► 寫入 ...（效能跌落數倍）
```

### 普遍性：rustc 自身的 20% 分支

這不是極端邊緣情況。根據編譯器分析統計，**Rust 編譯器自身（`rustc`）產生的二進位碼中，有高達 20% 的條件分支全都是邊界檢查**。

如何消除這些非必要檢查，是釋放 Rust 隱藏效能的關鍵戰場。

## 不碰 unsafe 榨乾效能：五大核心技巧
Duration: 15

我們完全不需要藉由 `unsafe` 關閉檢查。只要改變編碼習慣，給 LLVM 優化器足夠的靜態資訊，編譯器就會自動證明存取的絕對安全，並主動剔除迴圈內部的所有邊界檢查！

### 技巧一：預先切片（Pre-slicing）

**核心思維**：在進入迴圈之前，先建立一個已知長度的子切片。這相當於在「大門外一次性驗票」，進門之後的所有存取編譯器都能百分之百證明合法，內部檢查全部自動抹除。

```rust
fn sum_with_preslice(data: &[u32], len: usize) -> u32 {
    // 預先切片：一次性在進入迴圈前完成邊界校驗
    if len <= data.len() {
        let valid_slice = &data[..len];
        // valid_slice 保證索引皆在範圍內，迴圈內不再有任何邊界檢查
        valid_slice.iter().sum()
    } else {
        0
    }
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    assert_eq!(sum_with_preslice(&numbers, 3), 6);
}

#[test]
fn test_preslice() {
    let numbers = [10, 20, 30, 40];
    assert_eq!(sum_with_preslice(&numbers, 2), 30);
    assert_eq!(sum_with_preslice(&numbers, 10), 0);
}
```

### 技巧二：善用迭代器（Iterators）

**核心思維**：不要寫 C 風格的索引走訪 `for i in 0..len { arr[i] }`。迭代器天生封裝了容器的頭尾指標與邊界資訊，編譯器天然知道遍歷何時終止，因此根本不會生成任何邊界檢查代碼。

```rust
// 反例：以索引高頻遍歷，可能引發逐次邊界檢查並阻礙向量化
fn calculate_slow(data: &[i32], out: &mut [i32]) {
    for i in 0..data.len() {
        out[i] = data[i] * 2;
    }
}
```

改用慣用的迭代器寫法，效能不僅追平甚至超越手寫 C：

```rust
fn calculate_fast(data: &[i32], out: &mut [i32]) {
    // 使用 zip 同時迭代輸入與輸出，迭代器自動保證雙方邊界安全
    for (d, o) in data.iter().zip(out.iter_mut()) {
        *o = *d * 2;
    }
}

fn main() {
    let input = [1, 2, 3, 4];
    let mut output = [0; 4];
    calculate_fast(&input, &mut output);
    assert_eq!(output, [2, 4, 6, 8]);
}

#[test]
fn test_calculate_fast() {
    let input = [10, 20, 30];
    let mut output = [0; 3];
    calculate_fast(&input, &mut output);
    assert_eq!(output, [20, 40, 60]);
}
```

### 技巧三：手動斷言外提（Assert Hoisting / 給編譯器透底）

**核心思維**：在密集運算或固定步長的區塊前，手動加上 `assert!`。這就像直接給 LLVM 遞了一份「小抄」。編譯器利用值範圍傳播（Value Range Propagation），推導出後續的索引存取 100% 成立，從而完全剔除隨後的多次檢查。

```rust
fn process_quad(data: &[i32], out: &mut [i32]) {
    // 斷言外提：在大門口一次繳清檢查成本
    assert!(data.len() >= 4);
    assert!(out.len() >= 4);

    // LLVM 確認索引 0..4 必定在合法長度之內，底層組語將是連續純粹的 mov 與 add 指令
    out[0] = data[0] * 2;
    out[1] = data[1] * 2;
    out[2] = data[2] * 2;
    out[3] = data[3] * 2;
}

fn main() {
    let input = [1, 2, 3, 4];
    let mut output = [0; 4];
    process_quad(&input, &mut output);
    assert_eq!(output, [2, 4, 6, 8]);
}

#[test]
fn test_process_quad() {
    let input = [10, 20, 30, 40, 50];
    let mut output = [0; 5];
    process_quad(&input, &mut output);
    assert_eq!(output[..4], [20, 40, 60, 80]);
}
```

### 技巧四：多切片關聯時明確框死邊界（Zip 限制）

**核心思維**：當同時遍歷多個陣列計算（例如點積、矩陣運算）時，切忌分別用索引 `a[i]` 與 `b[i]` 存取。使用 `.zip()` 或以最小長度截取切片，編譯器就能將兩者的長度下限綁定，徹底消除交叉檢查。

```rust
fn dot_product(a: &[i32], b: &[i32]) -> i32 {
    // zip 巧妙地取兩者長度的最小值，循環過程不需要任何 index 邊界判定
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn main() {
    let v1 = [1, 2, 3];
    let v2 = [4, 5, 6];
    assert_eq!(dot_product(&v1, &v2), 32);
}

#[test]
fn test_dot_product() {
    assert_eq!(dot_product(&[2, 3], &[4, 5]), 23);
}
```

### 技巧五：固定區塊分批處理（as_chunks / SIMD 對齊）

**核心思維**：使用 `as_chunks::<N>()`（或動態步長的 `chunks_exact(n)`）將切片切割為編譯期靜態已知大小的陣列切片 `(&[[T; N]], &[T])`。編譯器確知每批恰好有 N 個元素且索引絕對合法，能毫無顧忌地將其完全向量化，剩餘未對齊元素則由 `remainder` 統一收尾。

```rust
fn sum_by_chunks(data: &[u32]) -> u32 {
    let mut total = 0;
    // 每次恰好取出 4 個元素的固定長度陣列，讓編譯器毫不猶豫生成 SIMD 向量加法
    let (chunks, remainder) = data.as_chunks::<4>();

    for chunk in chunks {
        total += chunk[0] + chunk[1] + chunk[2] + chunk[3];
    }
    for &val in remainder {
        total += val;
    }
    total
}

fn main() {
    let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(sum_by_chunks(&values), 55);
}

#[test]
fn test_sum_by_chunks() {
    let v = [10; 10];
    assert_eq!(sum_by_chunks(&v), 100);
}
```

Positive
: 上述五種技巧完全不需要 unsafe，全部依賴標準庫的型別保證與編譯器優化慣用法。透過正確的編碼架構，流失的 CPU 週期幾乎能 100% 被追回。

## Rust 與 C++ 的安全效能損耗對比
Duration: 7

### 共用同一個底層優化器：LLVM

Rust 與 C++ 都是以極致執行期效能為目標的系統級語言。扒掉外層語法，兩者底層均以 LLVM 作為主力後端（LLVM Backend & CodeGen）。在產生組合語言、暫存器分配、指令重排與常數傳播方面，兩者面對的是完全相同的物理法則與優化難題。

```
       C++ 源碼 (clang++)                 Rust 源碼 (rustc)
               │                                  │
       Clang AST / IR 生成                Rustc HIR / MIR 生成
               │                                  │
               └───────────────┬──────────────────┘
                               ▼
                        LLVM IR 中間表示
                               │
               ┌───────────────┴───────────────┐
               ▼                               ▼
       迴圈向量化與指令重排               死碼消除與內聯
               │                               │
               └───────────────┬───────────────┘
                               ▼
                       機器碼 (Target ASM)
```

### 檢查量的懸殊差距

當兩者都開啟安全加固機制時，背後的實際保護量卻大相逕庭：

- **C / C++ 的加固困境**：C/C++ 中指標衰退（Pointer Decay）極為普遍。裸指標無法攜帶長度資訊，因此即便開啟 `-fsanitize=bounds` 或使用容器加固，編譯器能執行的動態檢查量**通常只有 Rust 的五分之一左右**。
- **Rust 的全面覆蓋**：Rust 的切片原生為寬指標（Fat Pointer，包含指標位址與長度），所有切片索引存取均在編譯器監督之下。

### 效能損耗總帳：兩者近乎持平

經過各大基準測試（Benchmarks Game 及實務網路伺服器評測）的綜合統計：

| 評測維度 | Rust（Safe 預設） | C++（開啟對等安全加固） |
|---|---|---|
| 總體平均效能損耗 | **3% ~ 4%** | **3% ~ 4%** |
| 極端高頻索引微基準 | ~8% - 10% | ~8% - 10% |
| 安全檢查覆蓋率 | **100% 完整覆蓋** | 約 20%（受限於裸指針衰退） |
| 是否需要犧牲代碼結構 | 否（慣用迭代器自然消除） | 是（需改寫為專有容器或巨集） |

算總帳的時候，只要把記憶體安全放在第一位，不論使用哪一種程式語言，底層都要承擔相差無幾的物理成本。但 Rust 的革命性在於：**它在達到同等安全損耗的同時，提供了 5 倍以上的安全覆蓋率！**

## 獨家效能大殺器：別名分析（Aliasing Analysis）
Duration: 8

如果說邊界檢查是安全的支出，那麼**借用檢查器（Borrow Checker）則是 Rust 賦予開發者的獨家效能紅利**！

### 什麼是指針別名？C/C++ 的世界級難題

在編譯器最佳化領域，「別名分析（Aliasing Analysis）」是指判斷兩個指針是否可能指向同一塊記憶體區域。

在 C 與 C++ 中，兩個指標可能神不知鬼不覺地重疊：

```c
// C 語言經典場景：編譯器無法確認 out 與 in 是否指向同一陣列
void multiply(int* in, int* out, int count) {
    for (int i = 0; i < count; i++) {
        // 由於 out 可能與 in 重疊，寫入 *out 可能改變 in[i+1] 的值！
        // 編譯器不敢將 in 的資料快取在暫存器，每次迴圈都必須從記憶體重新讀取
        out[i] = in[i] * 2;
    }
}
```

為了解決這個問題，C99 引進了 `restrict` 關鍵字，但它全靠工程師口頭保證。一旦工程師看走眼讓指標重疊，就會引發毀滅性的未定義行為（UB），因此在實際大型工程中極少被普遍且安全地應用。

### 借用檢查器變身「效能保護神」

許多新手視為夢魘的 Rust 借用規則，到了編譯器優化階段直接化身效能大殺器：

- **`&mut T` 的本質就是「排他性唯一存取」（Exclusive Reference）**。在生命週期範圍內，絕對不可能存在任何其他指針能讀取或寫入這塊記憶體。
- 這項保證在編譯時期由借用檢查器 100% 證明，無需任何執行期檢查！
- Rust 編譯器因此可以毫無顧慮地為 LLVM IR 標註 **`noalias` 元數據**。

```rust
fn add_assign_vectors(dst: &mut [i32], src: &[i32]) {
    // 借用檢查器在編譯期百分之百確認 dst 與 src 記憶體互不重疊
    // LLVM 獲得 noalias 標註，能直接將資料拉至暫存器、合併讀寫並重排指令
    assert_eq!(dst.len(), src.len());
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d += *s;
    }
}

fn main() {
    let mut target = [1, 2, 3];
    let source = [10, 20, 30];
    add_assign_vectors(&mut target, &source);
    assert_eq!(target, [11, 22, 33]);
}

#[test]
fn test_add_assign() {
    let mut target = [5, 5];
    let source = [1, 2];
    add_assign_vectors(&mut target, &source);
    assert_eq!(target, [6, 7]);
}
```

有了 `noalias`，LLVM 擁有了完美的底層視野：
1. 大膽把記憶體變數提升（Promote）並長期存放在暫存器中。
2. 消除多餘的冗餘 Load / Store 指令。
3. 放開手腳進行自動向量化展開與指令管線排程。

這是 C++ 在保證安全前提下極難普遍享有的效能優勢。**借用檢查器不僅守護了安全，更是 Rust 獨步天下的效能紅利來源。**

## 總結：把流失的 CPU 週期全部拿回來
Duration: 4

### 效能優化心智模型

你不需要為了極致效能而背棄記憶體安全。在絕大多數情況下，寫出緩慢的 Rust 程式碼不是因為安全機制太笨，而是因為我們帶著舊語言的思維在寫 Rust。

```
+-------------------------------------------------------------+
|                  Safe Rust 高效編程自查清單                  |
+-------------------------------------------------------------+
| 1. 拒絕裸索引存取：走訪資料優先使用 Iterator 與組合子。     |
| 2. 多陣列同步處理：一律使用 .zip() 或統一以最小長度截取。    |
| 3. 固定小步長密集運算：在進入熱點迴圈前，手動將 assert 外提。 |
| 4. 連續批次資料運算：善用 as_chunks 或 chunks_exact 激活硬體 SIMD。 |
| 5. 善用 &mut 獨占特性：享受編譯器自帶的 noalias 暫存器紅利。 |
+-------------------------------------------------------------+
```

### 如何驗證邊界檢查是否被消除？

想知道你的程式碼是否成功消除了邊界檢查，可以使用 Cargo 外掛工具檢查組譯碼：

```console
cargo install cargo-show-asm
cargo asm --lib my_crate::my_function
```

在輸出的組合語言中：
- 若看到 `panic_bounds_check`、`call core::panicking::panic` 或額外的 `cmp / jge` 分支，表示邊界檢查仍殘留。
- 若看到緊湊的 `vmovdqu`、`vpaddd` 等向量化指令且無條件跳轉分支，代表編譯器已成功將邊界檢查完全消除並實施了 SIMD 向量化！

下次在面對百萬次密集迴圈時，先把這些 Safe Rust 的優化技巧用上一遍，你就能在死守安全底線的同時，把原本流失的 CPU 週期全部拿回來！
