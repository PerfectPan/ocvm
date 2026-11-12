FROM node:26.13.1-bookworm AS node
FROM rust:1-bookworm

COPY --from=node /usr/local/ /usr/local/

WORKDIR /work
COPY . .

ENV OCVM_HOME=/tmp/ocvm-home
ENV OCVM_NPM_PACKAGE=openclaw

RUN cargo build --release --locked
RUN node --version && npm --version

CMD ["./scripts/e2e-npm-openclaw.sh"]
