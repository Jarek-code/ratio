FROM rust:1.93-slim AS build

WORKDIR /app
COPY . .
# Las consultas de SQLx se validan con .sqlx/ (cargo sqlx prepare), sin base de datos al compilar
ENV SQLX_OFFLINE=true
RUN cargo build --release -p servidor

FROM debian:bookworm-slim

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /app/target/release/servidor /usr/local/bin/servidor

USER 10001:10001
ENV IP=0.0.0.0
ENV PORT=8080
EXPOSE 8080
CMD ["servidor"]
