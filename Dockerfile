FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev curl bzip2
WORKDIR /usr/src/covdir-report-action

# Download grcov for the current architecture
ARG GRCOV_VERSION=0.10.5
RUN curl -sL "https://github.com/mozilla/grcov/releases/download/v${GRCOV_VERSION}/grcov-$(uname -m)-unknown-linux-musl.tar.bz2" \
    | tar xjf - -C /usr/local/bin

COPY . .
RUN cargo install --path .

FROM alpine:3.22
# Install LLVM tools with zlib support for compressed profile data
RUN apk add --no-cache llvm18 \
    && ln -s /usr/lib/llvm18/bin/llvm-profdata /usr/local/bin/llvm-profdata \
    && ln -s /usr/lib/llvm18/bin/llvm-cov /usr/local/bin/llvm-cov
COPY --from=builder /usr/local/bin/grcov /usr/local/bin/grcov
COPY --from=builder /usr/local/cargo/bin/covdir-report-action /usr/local/bin/covdir-report-action
# Default LLVM tools path (can be overridden)
ENV LLVM_PATH=/usr/local/bin
ENTRYPOINT ["/usr/local/bin/covdir-report-action"]
