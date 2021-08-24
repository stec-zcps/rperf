FROM rust:1.54-slim

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["rperf"]
