FROM debian:stretch-slim as builder

ARG VERSION=0.4.2

RUN set -ex \
	&& apt-get update \
	&& apt-get install -qq --no-install-recommends ca-certificates wget \
	&& cd /tmp \
	&& wget -qO ord.tar.gz "https://github.com/casey/ord/releases/download/${VERSION}/ord-${VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
  && ls -la \
	&& mkdir bin \
	&& tar -xzvf ord.tar.gz -C /tmp/bin "ord"

FROM debian:stretch-slim
COPY --from=builder "/tmp/bin" /usr/local/bin

ENTRYPOINT [ "ord" ]


