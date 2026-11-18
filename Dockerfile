# syntax=docker/dockerfile:1
# 多階段建置：編譯器只存在於 builder 階段，最終映像只含執行檔。
# 動態連結 glibc，執行階段使用 distroless/cc。

FROM rust:1.99 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
# cache mount 讓依賴下載與編譯結果在多次 docker build 之間保留（類似 Go 的 module cache）。
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release --locked \
    && cp target/release/rust_practice /rust_practice

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /rust_practice /rust_practice
ENTRYPOINT ["/rust_practice"]
