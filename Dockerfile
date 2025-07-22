FROM buildpack-deps:jammy as rustbuild

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.86.0

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
    amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='5cc9ffd1026e82e7fb2eec2121ad71f4b0f044e88bca39207b3f6b769aaa799c' ;; \
    armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='48c5ecfd1409da93164af20cf4ac2c6f00688b15eb6ba65047f654060c844d85' ;; \
    arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='e189948e396d47254103a49c987e7fb0e5dd8e34b200aa4481ecc4b8e41fb929' ;; \
    i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='0e0be29c560ad958ba52fcf06b3ea04435cb3cd674fbe11ce7d954093b9504fd' ;; \
    *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.25.1/${rustArch}/rustup-init"; \
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
WORKDIR /usr/src/pff-tools/pff-cli
RUN cargo install --path .

# build web app
FROM node:23-slim AS nodebuild
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
COPY --from=rustbuild /usr/local/cargo/bin/pff-cli /app/pff-cli

ENV LD_LIBRARY_PATH=/app

EXPOSE 8800

CMD ["/app/pff-web"]
