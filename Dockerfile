# Build stage. Debian rather than Alpine on purpose: the rustls crypto
# provider compiles C, and glibc keeps that boring.
FROM rust:1.98-bookworm AS builder

WORKDIR /build

# Cache dependency compilation separately from source changes.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    echo '' > src/lib.rs && \
    cargo build --release --locked && \
    rm -rf src

COPY src ./src
# Touch so cargo does not reuse the dummy build artifacts.
RUN touch src/main.rs src/lib.rs && \
    cargo build --release --locked

# Runtime stage. distroless/cc carries glibc and libgcc and nothing else.
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /build/target/release/puzzle-sheets /usr/local/bin/puzzle-sheets

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/puzzle-sheets"]
