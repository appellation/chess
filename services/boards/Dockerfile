FROM rust

WORKDIR /usr/chess

RUN apt-get update && apt-get install -y clang
COPY Cargo.lock Cargo.toml ./
COPY services/api ./services/api/
COPY services/boards ./services/boards/

WORKDIR /usr/chess/services/boards

RUN cargo install --path .

CMD ["boards"]
