FROM rust:1.45 AS builder

WORKDIR /tachyomancer
COPY Cargo.toml Cargo.lock ./
RUN sed -i 's/"tachyomancer",//' Cargo.toml
COPY tachy ./tachy
COPY tachyoscope ./tachyoscope
RUN cargo build --release -p tachy
RUN cargo build --release -p tachyoscope



FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update && \
    apt-get install -y ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER && \
    useradd -g $APP_USER $APP_USER && \
    mkdir -p ${APP}

COPY --from=builder /tachyomancer/target/release/tachyoscope ${APP}/tachyoscope
RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}
EXPOSE 8080/tcp
CMD ./tachyoscope --host=$(hostname -I) --port=8080
