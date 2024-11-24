# syntax=docker/dockerfile:1

# FROM jeanblanchard/alpine-glibc AS image

#FROM alpine AS image

FROM amazoncorretto:17-alpine3.20 AS image

LABEL maintainer="work.aab.25122001@gmail.com"
LABEL version="1.0"
LABEL description="Deployyy"

ENV RUST_LOG=info \
    RCON_RANGE=26000..27000 \
    PORT_RANGE=25000..63000 \
    TIMEOUT=120 \
    MODE=dev
EXPOSE ${PORT}

# RUN apk update && apk add curl
RUN mkdir ${DATA_FOLDER}
WORKDIR /app
COPY .env .env
COPY patch_server_props.sh patch.sh
COPY msrvmanager app
COPY static static
CMD ["./app"]