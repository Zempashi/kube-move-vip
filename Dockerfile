ARG RUST_VERSION=1.98

################################################################################
# Create a stage for building the application.
################################################################################

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

# Install host build dependencies.
RUN apk add clang lld musl-dev git

# Build the application.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/var/cache/cargo \
    CARGO_HOME=/var/cache/cargo cargo build --locked --release --target x86_64-unknown-linux-musl && \
    cp ./target/x86_64-unknown-linux-musl/release/kube-move-vip /bin/kube-move-vip

FROM gcr.io/distroless/static-debian13 AS final
#FROM debian:trixie-slim AS final

COPY --from=build /bin/kube-move-vip /bin/

CMD ["/bin/kube-move-vip"]
