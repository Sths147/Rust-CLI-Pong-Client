FROM alpine:3.20

RUN apk add --update --no-cache bash pkgconfig curl gcc musl-dev libx11-dev openssl-dev openssl-libs-static alpine-conf

COPY ./src /src
COPY ./Cargo.toml ./Cargo.toml
COPY ./entrypoint.sh /entrypoint.sh

ENTRYPOINT ["bash", "/entrypoint.sh"]
CMD ["cargo", "run", "--release", "localhost:8081"]