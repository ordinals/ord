FROM rust:1.67 as build

WORKDIR /usr/src/ord
COPY . .

RUN cargo install --path .

FROM busybox:1.36.0 as runtime
COPY --from=build /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=build /lib/x86_64-linux-gnu/libdl.so.2 /lib/x86_64-linux-gnu/libdl.so.2
COPY --from=build /usr/src/ord/target/release/ord /usr/local/bin/ord
CMD ord
