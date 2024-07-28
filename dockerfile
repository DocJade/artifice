# varible for the name of the app
ARG BINARY_NAME_DEFAULT=artifice

# we use muslrust as a builder to let us link everything together
FROM clux/muslrust:stable AS builder

# docker user, from the repo, not sure what it does hehehe -Docjade
RUN groupadd -g 10001 -r dockergrp && useradd -r -g dockergrp -u 10001 dockeruser

# reimport the app name
ARG BINARY_NAME_DEFAULT
ENV BINARY_NAME=$BINARY_NAME_DEFAULT

# Build dummy main with the project's Cargo lock and toml
# This is a docker trick in order to avoid downloading and building 
# dependencies when lock and toml not is modified.
COPY Cargo.lock .
COPY Cargo.toml .

RUN mkdir src \
&& echo "fn main() {print!(\"Dummy main\");} // dummy file" > src/main.rs
RUN set -x && cargo build --target x86_64-unknown-linux-musl --release
RUN ["/bin/bash", "-c", "set -x && rm target/x86_64-unknown-linux-musl/release/deps/${BINARY_NAME//-/_}*"]

# Now add the rest of the project and build the real main
COPY src ./src

RUN set -x && cargo build --target x86_64-unknown-linux-musl --release
RUN mkdir -p /build-out
RUN set -x && cp target/x86_64-unknown-linux-musl/release/$BINARY_NAME /build-out/

# Create debian based image
FROM debian:bookworm-slim

# reimport the app name
ARG BINARY_NAME_DEFAULT
ENV BINARY_NAME=$BINARY_NAME_DEFAULT

# what does this do? no idea.
ENV RUST_LOG="error,$BINARY_NAME=info"

# Copy the binary
COPY --from=builder /build-out/$BINARY_NAME /

# install ffmpeg

RUN apt update
RUN apt install ffmpeg -y
# cleanup left behind files
RUN apt clean
RUN rm -rf /var/cache/apt/archives /var/lib/apt/lists


# These are the enviroment varibles that need to be set.
# Discord bot token
ENV TOKEN=""
# FFMPEG hardware acceleration, defaults to `none`
ENV HW_ACCEL=""

# run the bot
CMD ["/artifice"]