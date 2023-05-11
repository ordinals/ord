FROM ubuntu

COPY ./release/ord /usr/local/bin/ord

EXPOSE 7003
ENTRYPOINT ["ord"]