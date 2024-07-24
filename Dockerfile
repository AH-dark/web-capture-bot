FROM rust:1 AS builder

WORKDIR /app

RUN apt update && \
    apt upgrade -y

COPY . .

# 编译应用程序
RUN cargo build --release

FROM browserless/chrome:1-puppeteer-21.9.0 AS runner

WORKDIR /app

COPY --from=builder /app/target/release/web-capture-bot .

CMD ["/app/web-capture-bot"]
