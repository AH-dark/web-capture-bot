FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt update && \
    apt upgrade -y

COPY . .

# 编译应用程序
RUN cargo build --release

FROM debian:bookworm AS runtime

WORKDIR /app

# Install Chrome
RUN apt update && \
    apt install -y wget gnupg ca-certificates \
RUN wget -q -O - https://dl-ssl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    echo "deb http://dl.google.com/linux/chrome/deb/ stable main" >> /etc/apt/sources.list.d/google-chrome.list && \
    apt update && \
    apt install -y google-chrome-stable

COPY --from=builder /app/target/release/web-capture-bot .

CMD ["/app/web-capture-bot"]
