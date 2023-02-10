FROM rust:latest as builder

ARG VERSION=0.4.2

RUN USER=root
RUN mkdir builder
WORKDIR /builder

RUN set -ex \
	&& apt update \
	&& apt install -y libssl-dev


RUN git clone https://github.com/casey/ord.git


RUN set -ex \
	&& cd ord \
	&& cargo clean \
	&& cargo build --release \
	&& ls -la

FROM debian:buster-slim

# Copy the compiled binaries into the new container.
COPY --from=builder /builder/target/release/ord /usr/local/bin

CMD ["ord"]



