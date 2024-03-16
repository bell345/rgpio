FROM ubuntu:buster AS build
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
    gcc-arm-linux-gnueabi

#ARG RASPBERRY_PI_TOOLS_COMMIT_ID=5caa7046982f0539cf5380f94da04b31129ed521
#RUN curl -sL https://github.com/raspberrypi/tools/archive/$RASPBERRY_PI_TOOLS_COMMIT_ID.tar.gz \
#    | tar xzf - -C /usr/local --strip-components=1 tools-${RASPBERRY_PI_TOOLS_COMMIT_ID}/arm-bcm2708

RUN wget https://github.com/Pro/raspi-toolchain/releases/latest/download/raspi-toolchain

ENV RUST_TARGET=arm-unknown-linux-gnueabi
ENV GCC_TARGET=arm-bcm2708-linux-gnueabi

# Need to add both these to PATH.
ENV PATH=/usr/local/arm-bcm2708/$GCC_TARGET/bin:$PATH
ENV PATH=/usr/local/arm-bcm2708/$GCC_TARGET/libexec/gcc/$GCC_TARGET/4.8.3:$PATH

# Install Rust.
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --verbose
ENV PATH=/root/.cargo/bin:$PATH

# Install the arm target for Rust.
RUN rustup target add $RUST_TARGET
# Configure the linker for the arm target.
RUN echo "[target.$RUST_TARGET]\n" >> /root/.cargo/config
RUN echo "linker = \"$GCC_TARGET-gcc\"\n" >> /root/.cargo/config
RUN echo ""

ENV USER=root
WORKDIR /usr/src

COPY . .

CMD ["/bin/bash"]

#RUN cargo build -r --target=$RUST_TARGET

#ARG DEPLOY_HOST=gdoor
#ARG DEPLOY_PATH=/srv/http/rgpio
#ARG DEPLOY_USER=rgpio

#RUN --mount=type=secret,id=deploy_key,required=true \
#    scp \
#    -i /run/secrets/deploy_key/deploy_key \
#    target/$RUST_TARGET/release/rgpio \
#    "${DEPLOY_USER}@${DEPLOY_HOST}:${DEPLOY_PATH}"

