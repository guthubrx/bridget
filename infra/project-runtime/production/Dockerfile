# Image de développement générique, Linux amd64 uniquement. La base est
# épinglée par digest afin qu'un tag amont ne puisse pas changer l'image sans
# modification versionnée de ce fichier et de toolchain.lock.
FROM debian@sha256:d7e12182ce18b85b93007c1dedf31f2d29e01ccf3182cc4017c709b6259bc132

ARG PNPM_VERSION=10.15.0
ARG DEBIAN_FRONTEND=noninteractive
ARG TARGETARCH

LABEL org.opencontainers.image.title="Bridget project runtime" \
      org.opencontainers.image.description="Runtime de développement neutre, sans secret ni client fournisseur" \
      org.opencontainers.image.vendor="Bridget"

RUN test "$TARGETARCH" = "amd64" \
 && apt-get update \
 && apt-get install --yes --no-install-recommends \
      bash build-essential ca-certificates curl git jq nodejs npm rustup \
      python3 python3-venv ripgrep tini \
 && npm install --global "pnpm@${PNPM_VERSION}" \
 && install --directory --mode=0755 /opt/cargo /opt/rustup \
 && CARGO_HOME=/opt/cargo RUSTUP_HOME=/opt/rustup rustup toolchain install 1.88.0 --profile minimal \
 && CARGO_HOME=/opt/cargo RUSTUP_HOME=/opt/rustup rustup default 1.88.0 \
 && chmod -R a+rX /opt/cargo /opt/rustup \
 && rm -rf /var/lib/apt/lists/* /root/.npm

RUN groupadd --gid 1002 bridget \
 && useradd --uid 1002 --gid bridget --create-home --shell /bin/bash bridget \
 && install --directory --owner=bridget --group=bridget --mode=0700 \
      /var/lib/bridget-project/home \
      /var/lib/bridget-project/xdg/config \
      /var/lib/bridget-project/xdg/cache \
      /var/lib/bridget-project/xdg/data \
      /var/lib/bridget-project/xdg/state \
      /run/bridget/runtime

# Le lanceur est compilé depuis le checkout attesté passé comme contexte de
# build. Les agents ne reçoivent donc pas un faux chemin de binaire : la
# politique peut référencer /usr/local/bin/bridget de manière vérifiable.
# Le contexte est le sommet du dépôt, jamais ce sous-répertoire seul.
COPY Cargo.toml Cargo.lock /opt/bridget-src/
COPY crates /opt/bridget-src/crates
COPY plugins /opt/bridget-src/plugins
RUN cd /opt/bridget-src \
 && CARGO_HOME=/opt/cargo RUSTUP_HOME=/opt/rustup PATH=/opt/cargo/bin:$PATH \
      cargo build --release --locked -p bridget-daemon --bin bridget \
 && install --owner=root --group=root --mode=0755 \
      target/release/bridget /usr/local/bin/bridget \
 && rm -rf /opt/bridget-src/target

COPY --chmod=0755 infra/project-runtime/production/smoke.sh /usr/local/bin/bridget-runtime-smoke
COPY --chmod=0755 infra/project-runtime/production/idle.sh /usr/local/bin/bridget-runtime-idle

ENV HOME=/var/lib/bridget-project/home \
    XDG_CONFIG_HOME=/var/lib/bridget-project/xdg/config \
    XDG_CACHE_HOME=/var/lib/bridget-project/xdg/cache \
    XDG_DATA_HOME=/var/lib/bridget-project/xdg/data \
    XDG_STATE_HOME=/var/lib/bridget-project/xdg/state \
    CARGO_HOME=/opt/cargo \
    RUSTUP_HOME=/opt/rustup \
    PATH=/usr/local/bin:/opt/cargo/bin:/usr/bin:/bin

USER bridget:bridget
WORKDIR /workspace
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/bridget-runtime-idle"]
