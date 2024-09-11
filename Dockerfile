FROM debian:bookworm-slim

COPY target/release/mensa-api /

CMD [ "/mensa-api" ]
