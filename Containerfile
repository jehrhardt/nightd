FROM docker.io/library/rust:1.100-alpine3.24@sha256:8d0e82abfdf9a38071e07cabc5afba4ddbe077631a80e4e742a760c38beadc33 AS build

WORKDIR /src

COPY Cargo.toml Cargo.lock ./
COPY apps ./apps
COPY crates ./crates
RUN cargo build --release --locked -p nightd -p nightctl

FROM docker.io/library/alpine:3.24@sha256:f59abe656b692d901e184be3445e084786602108599d2a51d229c0f5a48c71bf AS base

RUN apk add --no-cache ca-certificates tzdata \
    && addgroup -g 1000 -S nightd \
    && adduser -u 1000 -S -G nightd -h /home/nightd -s /sbin/nologin nightd \
    && mkdir -p /run/nightd \
    && chown nightd:nightd /run/nightd \
    && chmod 700 /run/nightd

FROM scratch AS release-base

COPY --from=base /etc/passwd /etc/passwd
COPY --from=base /etc/group /etc/group
COPY --from=base /usr/share/zoneinfo /usr/share/zoneinfo
COPY --from=base /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

FROM release-base AS release-nightd

COPY --from=base /run /run

ENTRYPOINT ["/usr/local/bin/nightd"]

COPY --from=build /src/target/release/nightd /usr/local/bin/nightd

USER nightd:nightd

FROM release-base AS release-nightctl

ENTRYPOINT ["/usr/local/bin/nightctl"]

COPY --from=build /src/target/release/nightctl /usr/local/bin/nightctl

USER nightd:nightd
