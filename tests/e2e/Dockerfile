FROM rust:1-bookworm

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates curl gnupg \
  && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
  && apt-get install -y --no-install-recommends nodejs \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /work
COPY . .

ENV OCVM_HOME=/tmp/ocvm-home
ENV OCVM_NPM_PACKAGE=openclaw

RUN cargo build --release --locked
RUN node --version && npm --version

CMD ["./scripts/e2e-npm-openclaw.sh"]
