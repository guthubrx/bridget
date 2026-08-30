FROM debian@sha256:d7e12182ce18b85b93007c1dedf31f2d29e01ccf3182cc4017c709b6259bc132

RUN groupadd --gid 1002 bridget && useradd --uid 1002 --gid bridget --create-home --shell /usr/sbin/nologin bridget && \
    mkdir -p /var/lib/bridget-project/home \
      /var/lib/bridget-project/xdg/config \
      /var/lib/bridget-project/xdg/cache \
      /var/lib/bridget-project/xdg/data \
      /var/lib/bridget-project/xdg/state \
      /run/bridget/runtime && \
    chown -R 1002:1002 /var/lib/bridget-project /run/bridget

# Contexte additionnel injecté uniquement par le test d'intégration SPEC-066.
# Cette image ne fait pas partie du registre d'agents ni d'une politique hôte.
COPY --from=bridget-bin bridget /usr/local/bin/bridget
COPY fixture-agent /usr/local/bin/fixture-agent
RUN chmod 0755 /usr/local/bin/bridget /usr/local/bin/fixture-agent
