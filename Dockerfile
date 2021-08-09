FROM rust:1.53-slim

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

CMD ["rperf"]