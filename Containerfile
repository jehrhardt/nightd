# syntax=docker/dockerfile:1

FROM docker.io/library/rust:1.100-alpine3.24 AS build

RUN apk add --no-cache ca-certificates tzdata \
    && addgroup -g 10001 -S nonroot \
    && adduser -u 10001 -S -G nonroot -h /home/nonroot -s /sbin/nologin nonroot

WORKDIR /src

COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
RUN cargo build --release --locked -p nightd -p nightctl

FROM scratch AS release-base

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group
COPY --from=build /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

FROM release-base AS release-nightd

ENTRYPOINT ["/usr/local/bin/nightd"]

COPY --from=build /src/target/release/nightd /usr/local/bin/nightd

USER nonroot:nonroot

FROM release-base AS release-nightctl

ENTRYPOINT ["/usr/local/bin/nightctl"]

COPY --from=build /src/target/release/nightctl /usr/local/bin/nightctl

USER nonroot:nonroot
