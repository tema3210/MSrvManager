# syntax=docker/dockerfile:1

FROM jeanblanchard/alpine-glibc AS image

LABEL maintainer="work.aab.25122001@gmail.com"
LABEL version="1.0"
LABEL description="Deployyy"

ENV PORT=80 \
    ADDR=10.243.254.7 \
    RUST_LOG=info \
    DATA_FOLDER=/minecraftServers
EXPOSE 80

RUN apk update && apk add libpq
WORKDIR /app
COPY .env .env
COPY static static
COPY msrvmanager app
CMD ["./app"]