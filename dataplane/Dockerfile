FROM archlinux as builder

RUN pacman -Syu --noconfirm
RUN pacman -S base-devel protobuf rustup --noconfirm

RUN rustup default stable
RUN rustup install nightly
RUN rustup component add rust-src --toolchain nightly
RUN rustup target add x86_64-unknown-linux-musl
RUN --mount=type=cache,target=/root/.cargo/registry \
    cargo install bpf-linker

WORKDIR /workspace

COPY . .
RUN --mount=type=cache,target=/workspace/target/ \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo xtask build-ebpf --release
RUN --mount=type=cache,target=/workspace/target/ \
    --mount=type=cache,target=/root/.cargo/registry \
    RUSTFLAGS=-Ctarget-feature=+crt-static cargo build --release --target=x86_64-unknown-linux-musl
RUN --mount=type=cache,target=/workspace/target/ \
    cp /workspace/target/x86_64-unknown-linux-musl/release/loader /workspace/dataplane

FROM alpine

LABEL org.opencontainers.image.source=https://github.com/kubernetes-sigs/blixt
LABEL org.opencontainers.image.licenses=GPL-2.0-only,BSD-2-Clause

WORKDIR /opt/blixt/

COPY --from=builder /workspace/dataplane /opt/blixt/dataplane

COPY LICENSE.GPL-2.0 /opt/blixt/LICENSE.GPL-2.0
COPY LICENSE.BSD-2-Clause /opt/blixt/LICENSE.BSD-2-Clause

ENTRYPOINT ["/opt/blixt/dataplane"]
