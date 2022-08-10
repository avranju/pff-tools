FROM buildpack-deps:jammy as rustbuild

# install rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.62.1

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='3dc5ef50861ee18657f9db2eeb7392f9c2a6c95c90ab41e45ab4ca71476b4338' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='67777ac3bc17277102f2ed73fd5f14c51f4ca5963adadf7f174adf4ebc38747b' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='32a1532f7cef072a667bac53f1a5542c99666c4071af0c9549795bbdb2069ec1' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='e50d1deb99048bc5782a0200aa33e4eea70747d49dffdc9d06812fd22a372515' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.24.3/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

# build libpff
RUN apt update && apt install -y git autoconf automake autopoint \
    libtool pkg-config llvm-dev libclang-dev clang
WORKDIR /usr/src
RUN git clone https://github.com/libyal/libpff.git
WORKDIR /usr/src/libpff
RUN ./synclibs.sh && ./autogen.sh && ./configure && make && make install
RUN ldconfig

# build pff-web
WORKDIR /usr/src/pff-tools
COPY . .
WORKDIR /usr/src/pff-tools/pff-web
RUN cargo install --path .

# build web app
FROM node:18 as nodebuild
WORKDIR /usr/src/pff-tools
COPY . .
WORKDIR /usr/src/pff-tools/pff-web
RUN yarn install
RUN yarn run build

# build final image
FROM ubuntu:jammy

# install dependencies
RUN apt update && \
    apt install ca-certificates libssl3 -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=nodebuild /usr/src/pff-tools/pff-web/www /app/www
COPY --from=rustbuild /usr/local/lib/libpff.so.1 /app/libpff.so.1
COPY --from=rustbuild /usr/local/cargo/bin/pff-web /app/pff-web

ENV LD_LIBRARY_PATH=/app

EXPOSE 8800

CMD ["/app/pff-web"]