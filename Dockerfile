FROM rust:1.92-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/covdir-report-action
COPY . .
RUN cargo install --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/covdir-report-action /usr/local/bin/covdir-report-action
ENTRYPOINT [ "/usr/local/bin/covdir-report-action" ]
