FROM messense/rust-musl-cross:x86_64-musl as builder
WORKDIR /webrtc_stream_server

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /webrtc_stream_server/target/x86_64-unknown-linux-musl/release/webrtc_stream_server /webrtc_stream_server
ENTRYPOINT ["/webrtc_stream_server"]
EXPOSE 3536