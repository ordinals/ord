FROM rust:latest as builder

ARG VERSION=0.4.2

RUN USER=root
RUN mkdir builder
WORKDIR /builder

RUN set -ex \
	&& apt update \
	&& apt install -y libssl-dev \
	&& git clone https://github.com/casey/ord.git \
	&& cd ord \
	&& cargo clean \
	&& cargo build --release

FROM debian:buster-slim

# Copy the compiled binaries into the new container.
COPY --from=builder /builder/target/release/ord /usr/local/bin

CMD ["ord"]



