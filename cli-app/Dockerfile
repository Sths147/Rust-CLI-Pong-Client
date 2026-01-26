FROM alpine:3.20 AS dev

RUN apk add --update --no-cache pkgconfig 			\
                                curl 				\
                                gcc 				\
                                musl-dev 			\
                                libx11-dev 			\
                                openssl-dev 		\
                                openssl-libs-static \
                                xorg-server 		\
                                xf86-video-vesa

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./entrypoint.sh /entrypoint.sh

WORKDIR /app
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- -y 	&& \
	rustup update																&& \
	cargo build --release

FROM alpine:3.20 AS prod

COPY --from=dev app/target/release/transcendence_cli_app ./cli_app
RUN chmod +x ./cli_app

WORKDIR /app

CMD ["bash", "./entrypoint.sh"]