FROM rust:1-bookworm

RUN apt-get update \
  && apt-get install -y --no-install-recommends nodejs npm ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /work
COPY . .

ENV OCVM_HOME=/tmp/ocvm-home
ENV OCVM_NPM_PACKAGE=openclaw

RUN cargo build --release --locked

CMD ["./scripts/e2e-npm-openclaw.sh"]

