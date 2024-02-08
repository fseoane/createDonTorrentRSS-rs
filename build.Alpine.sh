sudo apk add pkgconfig
sudo apk add gcc musl-dev openssl openssl-dev
#  remember to add a dependency in cargo:
#       git2 = {version="0.13.22", features = ["vendored-libgit2"]}
RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build --release