FROM rust

WORKDIR /usr/chess

COPY Cargo.lock Cargo.toml ./
COPY services/api ./services/api/
COPY services/boards ./services/boards/

WORKDIR /usr/chess/services/api

RUN cargo install --path .

CMD ["api"]
