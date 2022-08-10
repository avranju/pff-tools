FROM debian:buster-slim

# install dependencies
RUN apt update && \
    apt install ca-certificates libssl1.1 -y

WORKDIR /app

COPY pff-web /app/pff-web
COPY www/ /app/www/

EXPOSE 8800

CMD ["/app/pff-web"]
