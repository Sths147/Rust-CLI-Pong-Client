xset r rate 100 30
docker build . -t cli
docker run -it cli:latest cargo run --release $1
xset r rate 500 30