curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs/ | sh -s -- -y
setup-xorg-base
source $HOME/.cargo/env
rustup update
exec "$@"