FROM --platform=$BUILDPLATFORM docker.io/library/rust:1 AS chef
WORKDIR /app
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
RUN apt update && apt install -y gcc-aarch64-linux-gnu
RUN cargo install cargo-chef
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
  "linux/amd64") echo x86_64-unknown-linux-musl > /rust_target.txt ;; \
  "linux/arm64/v8") echo aarch64-unknown-linux-musl > /rust_target.txt ;; \
  *) exit 1 ;; \
esac
RUN rustup target add $(cat /rust_target.txt)

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook \
--profile dist \
--target $(cat /rust_target.txt) \
--recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY src ./src 
RUN cargo build \
--profile dist \
--target $(cat /rust_target.txt)
RUN cp target/$(cat /rust_target.txt)/dist/podlet .

FROM scratch
COPY --from=builder /app/podlet /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/podlet" ]
