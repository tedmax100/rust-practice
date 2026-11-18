#!/usr/bin/env python3
"""檢查教學 Markdown 內的每一段 ```rust 範例，確保教學本身遵守 Google Rust Style Guide。

Go 範例（```go）以 gofmt -d 與 go vet 檢查。
每段 Rust 範例必須是完整、可獨立編譯的程式（含 `fn main`），並通過：
  1. rustfmt --check（讀取專案根目錄的 rustfmt.toml，max_width = 100）
  2. clippy-driver -D warnings，並啟用與 Cargo.toml [lints] 相同的規則
  3. rustc --test 編譯並執行其中的 #[test]

第一行含有「反例」的區塊是刻意示範錯誤寫法，會被略過。
第一行為「// excerpt: <路徑>」的 Rust 區塊是實際範例專案的節錄，只檢查其內容
與該檔案逐字相同（範例專案本身由 CI 以 fmt / clippy / test 檢查）。

用法：python3 scripts/check_snippets.py codelabs/tutorials/*.md
"""
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
FENCE = re.compile(r"^```(rust|go)\s*\n(.*?)^```", re.S | re.M)

# 與 Cargo.toml 的 [lints] 保持一致。
LINT_FLAGS = [
    "-D", "warnings",
    "-F", "unsafe_code",
    "-W", "clippy::wildcard_imports",
    "-W", "clippy::enum_glob_use",
    "-W", "clippy::unwrap_used",
    "-W", "clippy::fn_params_excessive_bools",
    "-W", "clippy::redundant_clone",
    "-W", "clippy::clone_on_ref_ptr",
    "-W", "clippy::missing_panics_doc",
    "-W", "clippy::missing_errors_doc",
    "-W", "clippy::len_without_is_empty",
    "-W", "clippy::new_without_default",
]


def run(cmd, cwd):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def check_block(source, workdir, label):
    errors = []
    src = workdir / "snippet.rs"
    src.write_text(source, encoding="utf-8")

    fmt = run(["rustfmt", "--check", "--edition", "2021",
               "--config-path", str(ROOT / "rustfmt.toml"), str(src)], workdir)
    if fmt.returncode != 0:
        errors.append(f"rustfmt 與標準格式不同：\n{fmt.stdout}{fmt.stderr}")

    lint = run(["clippy-driver", "--edition", "2021", "--test", "--crate-name", "snippet",
                "-o", str(workdir / "snippet_bin"), *LINT_FLAGS, str(src)], workdir)
    if lint.returncode != 0:
        errors.append(f"clippy / rustc：\n{lint.stderr}")
    else:
        test = run([str(workdir / "snippet_bin")], workdir)
        if test.returncode != 0:
            errors.append(f"測試失敗：\n{test.stdout}{test.stderr}")
    return [f"{label}: {e}" for e in errors]


def check_go(source, workdir, label):
    errors = []
    src = workdir / "snippet.go"
    src.write_text(source, encoding="utf-8")
    fmt = run(["gofmt", "-d", str(src)], workdir)
    if fmt.returncode != 0 or fmt.stdout.strip():
        errors.append(f"gofmt 與標準格式不同：\n{fmt.stdout}{fmt.stderr}")
    vet = run(["go", "vet", str(src)], workdir)
    if vet.returncode != 0:
        errors.append(f"go vet：\n{vet.stderr}")
    return [f"{label}: {e}" for e in errors]


def main(paths):
    problems, checked, skipped = [], 0, 0
    with tempfile.TemporaryDirectory() as tmp:
        workdir = pathlib.Path(tmp)
        for path in paths:
            text = pathlib.Path(path).read_text(encoding="utf-8")
            for match in FENCE.finditer(text):
                lang, code = match.group(1), match.group(2)
                line = text.count("\n", 0, match.start()) + 1
                if "反例" in code.split("\n", 1)[0]:
                    skipped += 1
                    continue
                first_line = code.split("\n", 1)[0]
                if lang == "rust" and first_line.startswith("// excerpt: "):
                    checked += 1
                    source = ROOT / first_line[len("// excerpt: "):].strip()
                    body = code.split("\n", 1)[1].rstrip("\n")
                    if not source.is_file():
                        problems.append(f"{path}:{line}: 找不到節錄來源 {source}")
                    elif body not in source.read_text(encoding="utf-8"):
                        problems.append(f"{path}:{line}: 節錄內容與 {source.name} 不一致")
                    continue
                checked += 1
                if lang == "go":
                    problems += check_go(code, workdir, f"{path}:{line}")
                    continue
                if "fn main" not in code:
                    problems.append(f"{path}:{line}: 範例必須包含 fn main 才能獨立編譯")
                    continue
                problems += check_block(code, workdir, f"{path}:{line}")
    for problem in problems:
        print(problem)
    print(f"snippets: checked {checked}, skipped {skipped} 反例, problems {len(problems)}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main(sys.argv[1:])
