FROM rust:1.70.0-alpine3.18 as builder
RUN apk add --no-cache build-base bash curl
ADD . .
RUN cargo build --release

FROM rust:1.70.0-alpine3.18
COPY --from=builder target/release/ord /usr/local/bin/ord
