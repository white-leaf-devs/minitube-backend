FROM rust as plan 
WORKDIR /app 
RUN cargo install cargo-chef
COPY . . 
RUN cargo chef prepare --recipe-path recipe.json

FROM rust as cache 
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=plan /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as build 
WORKDIR /app
COPY . . 
COPY --from=cache /app/target target
COPY --from=cache $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM amazon/aws-lambda-provided:al2
RUN yum install -y openssl11-libs
COPY --from=build /app/target/release/label-thumbnail ${LAMBDA_RUNTIME_DIR}/bootstrap
CMD [ "" ] 