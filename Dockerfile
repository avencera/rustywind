FROM rust:alpine as builder
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=builder /home/rust/src/target/release/rustywind .
USER 1000:1000
ENTRYPOINT ["./rustywind"]
