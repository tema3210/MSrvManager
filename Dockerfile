# syntax=docker/dockerfile:1

FROM jeanblanchard/alpine-glibc AS image

LABEL maintainer="work.aab.25122001@gmail.com"
LABEL version="1.0"
LABEL description="Deployyy"

ENV PORT=80 \
    ADDR=10.243.254.7 \
    RUST_LOG=info \
    RCON_RANGE=26000..27000 \
    DATA_FOLDER=/data
EXPOSE ${PORT}

# RUN apk update && apk add curl
RUN mkdir ${DATA_FOLDER}
WORKDIR /app
COPY .env .env
COPY static static
COPY msrvmanager app
CMD ["./app"]