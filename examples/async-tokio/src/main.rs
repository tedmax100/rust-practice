//! Go 並發模式在 tokio 上的對照範例：spawn、channel、select、取消與一次性初始化。

use std::sync::OnceLock;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinSet;
use tokio::time;
use tokio_util::sync::CancellationToken;

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

/// 對應 Go 中「只傳一次結果」的 channel。
async fn compute_in_background() -> u32 {
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        sender.send(6 * 7).expect("receiver 仍在等待");
    });
    receiver.await.expect("sender 不應該在送出前被丟棄")
}

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

/// 對應 Go 的 `sync.Once`：第一次呼叫時才初始化，之後重複使用。
fn app_name() -> &'static str {
    static APP_NAME: OnceLock<String> = OnceLock::new();
    APP_NAME.get_or_init(|| String::from("async-tokio-demo"))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn workers_all_finish() {
        assert_eq!(run_workers().await, vec![0, 10, 20], "三個 worker 的結果");
    }

    #[tokio::test]
    async fn channel_sums_values() {
        assert_eq!(sum_from_channel().await, 6, "1 + 2 + 3");
    }

    #[tokio::test]
    async fn oneshot_returns_value() {
        assert_eq!(compute_in_background().await, 42, "6 * 7");
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_only_when_too_slow() {
        let one_second = Duration::from_secs(1);
        let fast = fetch_with_timeout(Duration::from_millis(100), one_second).await;
        let slow = fetch_with_timeout(Duration::from_secs(2), one_second).await;
        assert_eq!(fast, Some("完成"), "快的工作應該在時限內完成");
        assert_eq!(slow, None, "慢的工作應該逾時");
    }

    #[tokio::test(start_paused = true)]
    async fn ticks_stop_after_cancel() {
        let received = count_ticks(Duration::from_millis(35)).await;
        assert_eq!(received, 3, "35 毫秒內應該收到 10、20、30 毫秒的三次計數");
    }

    #[test]
    fn app_name_is_initialized_once() {
        assert!(
            std::ptr::eq(app_name(), app_name()),
            "兩次呼叫應回傳同一份資料"
        );
    }
}
