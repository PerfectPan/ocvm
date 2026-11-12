FROM node:26-bookworm AS node
FROM rust:1-bookworm

COPY --from=node /usr/local/ /usr/local/
RUN dpkg --compare-versions "$(node -p 'process.versions.node')" ge 26.13.1 \
  && node --version && npm --version

WORKDIR /work
COPY . .

ENV OCVM_HOME=/tmp/ocvm-home
ENV OCVM_NPM_PACKAGE=openclaw

RUN cargo build --release --locked

CMD ["./scripts/e2e-npm-openclaw.sh"]
