FROM rust:1.70-bullseye

RUN set -x

RUN apt-get update \
    && apt-get -y upgrade \
    && apt-get -y install rsync

COPY install.sh .
RUN ./install.sh

RUN set +x

ENV CARGO_TERM_COLOR=always

CMD bash
