FROM debian:bullseye AS build
LABEL authors="Thomas Bell"

RUN apt-get update && \
    apt-get install \
    --yes \
    binutils \
    curl \
    zip \
    build-essential

# https://github.com/tiziano88/rust-raspberry-pi/blob/master/Dockerfile

RUN apt-get install --yes \
    gcc-arm-linux-gnueabihf

#ARG RASPBERRY_PI_TOOLS_COMMIT_ID=5caa7046982f0539cf5380f94da04b31129ed521
#RUN curl -sL https://github.com/raspberrypi/tools/archive/$RASPBERRY_PI_TOOLS_COMMIT_ID.tar.gz \
#    | tar xzf - -C /usr/local --strip-components=1 tools-${RASPBERRY_PI_TOOLS_COMMIT_ID}/arm-bcm2708

#RUN wget https://github.com/Pro/raspi-toolchain/releases/latest/download/raspi-toolchain

RUN curl -L "https://downloads.sourceforge.net/project/raspberry-pi-cross-compilers/Raspberry%20Pi%20GCC%20Cross-Compiler%20Toolchains/Bullseye/GCC%2010.2.0/Raspberry%20Pi%201%2C%20Zero/cross-gcc-10.2.0-pi_0-1.tar.gz" \
    | tar xzf - -C /usr/local --strip-components=1

ENV RUST_TARGET=arm-unknown-linux-gnueabihf
ENV GCC_TARGET=arm-linux-gnueabihf

# Need to add both these to PATH.
ENV PATH=/usr/local/bin:$PATH
ENV PATH=/usr/local/libexec/gcc/$GCC_TARGET/10.2.0:$PATH
#ENV PATH=/usr/local/arm-bcm2708/$GCC_TARGET/bin:$PATH
#ENV PATH=/usr/local/arm-bcm2708/$GCC_TARGET/libexec/gcc/$GCC_TARGET/4.8.3:$PATH

# Install Rust.
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --verbose
ENV PATH=/root/.cargo/bin:$PATH

# Install the arm target for Rust.
RUN rustup target add $RUST_TARGET
# Configure the linker for the arm target.
RUN echo "[target.$RUST_TARGET]\n" >> /root/.cargo/config
RUN echo "linker = \"$GCC_TARGET-gcc\"\n" >> /root/.cargo/config

ENV USER=root
WORKDIR /usr/src

# build dependencies before actual code
COPY Cargo.toml Cargo.lock build.rs ./
RUN \
    mkdir src && \
    echo 'fn main() {}' > src/main.rs && \
    cargo build -r --target=$RUST_TARGET && \
    rm -Rvf src

# build actual code
COPY . .
RUN cargo build -r --target=$RUST_TARGET

VOLUME /mnt/export
CMD ["/bin/bash", "export.sh"]

#ARG DEPLOY_HOST=gdoor
#ARG DEPLOY_PATH=/srv/http/rgpio
#ARG DEPLOY_USER=rgpio

#RUN --mount=type=secret,id=deploy_key,required=true \
#    scp \
#    -i /run/secrets/deploy_key/deploy_key \
#    target/$RUST_TARGET/release/rgpio \
#    "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}"

