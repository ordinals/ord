FROM rust:latest as builder

ARG TAG=master

USER root
RUN mkdir builder
WORKDIR /builder

RUN set -ex \
	&& apt update \
	&& apt install -y libssl-dev


RUN git clone https://github.com/tiero/ord.git

RUN set -ex \
	&& cd ord \
  && git checkout $TAG \
	&& cargo clean \
	&& cargo build --release

FROM debian:bullseye-slim AS runtime
USER root
COPY --from=builder /builder/ord/target/release/ord /usr/local/bin
ENTRYPOINT ["/usr/local/bin/ord"]