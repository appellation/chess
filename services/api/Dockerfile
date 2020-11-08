FROM rust

WORKDIR /usr/chess

RUN apt-get update && apt-get install -y clang

COPY Cargo.lock Cargo.toml ./

WORKDIR /usr/chess/services/api
COPY services/api ./
RUN cargo install --no-default-features --path .

CMD ["api"]
