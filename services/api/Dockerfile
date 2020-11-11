FROM rust

WORKDIR /usr/chess

RUN apt-get update && apt-get install -y clang
RUN cargo install refinery_cli
COPY Cargo.lock Cargo.toml ./

WORKDIR /usr/chess/services/api

COPY services/api ./
ARG DATABASE_URL
RUN refinery migrate -e DATABASE_URL files
RUN cargo install --path .

CMD ["api"]
