FROM public.ecr.aws/docker/library/rust:1.81.0 as rust-builder

WORKDIR /rust/rust_app
COPY src/ /rust/rust_app/src/
COPY Cargo.toml /rust/rust_app/
COPY Cargo.lock /rust/rust_app/

RUN apt-get update
RUN apt-get install musl-tools -y
RUN rustup update && \
    rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM public.ecr.aws/lambda/provided:al2023

COPY --from=rust-builder /rust/rust_app/target/x86_64-unknown-linux-musl/release/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap
COPY --from=<Your AWS account id>.dkr.ecr.<Your AWS region>.amazonaws.com/lambda-extension/otel-collector:v1 /src/collector /opt/extensions/collector
COPY adot-config.yaml /opt/collector-config/config.yaml
CMD [ "lambda-handler" ]