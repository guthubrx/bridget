# Recherche et décisions: Environnement Docker partagé par projet

## Décision 1 - Un conteneur par projet, pas par agent

**Décision**: tous les agents d'un projet partagent un environnement. Les
worktrees séparent les travaux concurrents quand nécessaire.

**Rationale**: les agents doivent collaborer sur le même dépôt, partager les
outils et éviter la duplication des caches. Le projet est la frontière de
confiance utile aujourd'hui.

**Alternative considérée**: un conteneur systématique par agent.

**Rejet**: multiplication des environnements, montages, caches et transferts;
coordination de fichiers plus complexe sans besoin d'isolation intra-projet
prouvé.

**Exception future**: agent non fiable, privilège spécial ou secret distinct.
Cette exception exige une nouvelle spec.

## Décision 2 - Bridget et Maicie restent sur l'hôte

**Décision**: une seule instance de chaque automate sert tous les projets.

**Rationale**: Bridget possède déjà l'annuaire, la livraison et le cycle de vie;
Maicie possède les objectifs et décisions. Les dupliquer créerait plusieurs
sources de vérité et une fédération prématurée.

**Alternative considérée**: un stack Bridget/Maicie complet par conteneur.

**Rejet**: synchronisation, découverte, upgrades et reprise deviennent plus
complexes sans valeur immédiate.

## Décision 3 - Docker CLI avant un SDK

**Décision**: construire des commandes par arguments, sous politique fermée.

**Rationale**: `std::process::Command` et les timeouts existent déjà. Un SDK
Docker ajouterait une dépendance, un modèle async et une surface de maintenance.

**Alternative considérée**: API Docker Rust.

**Condition de réexamen**: au moins trois limitations réelles du CLI ou un
besoin de flux événementiel impossible à borner proprement.

## Décision 4 - Pas de secret dans la première tranche Docker

**Décision**: prouver le runtime avec un agent fixture avant d'exposer les
credentials.

**Rationale**: le cycle de vie et la propagation de secrets sont deux risques
indépendants. Les livrer ensemble rendrait les échecs et fuites difficiles à
attribuer.

## Décision 5 - Point d'entrée Unix dédié

**Décision**: Bridget expose un socket distinct par projet et génération dans
un répertoire privé ne contenant aucun autre état. Le socket est monté dans un
seul conteneur et le handshake atteste projet, liaison, conteneur, epoch et
génération Bridget avant tout effet.

**Alternative considérée**: monter `~/.cache/bridget`.

**Rejet**: tous les agents verraient noms, pid files et états d'autres projets.

**Alternative considérée**: port TCP sur le bridge Docker.

**Rejet**: nouvelle authentification réseau et surface d'écoute inutile.

**Alternative considérée**: un socket Unix commun à tous les projets.

**Rejet**: la possession du socket commun permettrait une présentation
trans-projet; les labels Docker ne prouvent pas la provenance de la connexion.

## Décision 6 - Réseau bridge, limite explicite

**Décision**: aucun port publié, pas de host network, mais egress bridge non
filtré finement en v1.

**Rationale**: les fournisseurs ont besoin du réseau. Une allowlist fiable
exige proxy, résolution DNS et maintenance qui dépassent le besoin actuel.

**Conséquence**: 066 améliore filesystem/process, pas la confidentialité réseau.

## Sources primaires

- OpenAI recommande une politique sandbox explicite et la maîtrise du réseau:
  https://openai.com/index/running-codex-safely/
- Claude Code fournit un devcontainer de référence et distingue conteneur et
  sandboxing de commandes:
  https://code.claude.com/docs/en/devcontainer
  https://code.claude.com/docs/en/sandboxing
- Cursor exécute ses background agents dans une machine isolée, ce qui confirme
  que l'environnement d'exécution est une frontière explicite:
  https://docs.cursor.com/background-agent
- Docker documente le mode rootless comme réduction de privilèges du daemon et
  des conteneurs:
  https://docs.docker.com/engine/security/rootless/
- Docker rappelle que les bind mounts donnent par défaut un accès en écriture à
  l'hôte, d'où la nécessité d'une liste fermée:
  https://docs.docker.com/engine/storage/bind-mounts/

## Red flags retenus

- Docker n'est pas une isolation parfaite contre un adversaire hostile.
- Monter le Docker socket annule la frontière.
- Monter le home global transfère presque tous les secrets et configurations.
- Un réseau bridge n'est pas un filtrage egress.
- Un conteneur partagé signifie que les agents du projet partagent la confiance.
- Une image taggée sans digest n'est pas reproductible.

## Complément de décision RC8 - 2026-08-30

- La politique runtime est un document hôte Bridget fermé et séparé de la
  requête projet. Son absence ferme Docker, pas le backend host.
- Le conteneur utilise un UID/GID numérique non-root, un HOME/XDG sous son
  unique state root et un socket wrapper explicite. La dérivation historique
  depuis HOME reste limitée au backend host.
- Une image locale est épinglée par son image ID `sha256` obtenu avec
  `docker build --iidfile`; une image de registre utilise
  `repository@sha256`. Aucun tag n'est autoritatif.
- Le daemon cible ne se déclare pas rootless. Cette limite reste dans le modèle
  de menace et ne justifie pas l'ajout d'un orchestrateur ou d'un second moteur.

## Complément après intégration de SPEC-068 - 2026-08-30

- Le canal durable d'incidents runtime délégués existe déjà dans
  `idempotency.rs`, `fleet.rs`, `protocol.rs` et `daemon.rs`. Le backend Docker
  doit conserver ses codes, son ordre, son curseur, son rejeu et son
  acquittement au lieu de créer un chemin parallèle.
- La preuve de parité porte aussi sur `ProjectReference`: un changement de
  backend ne change pas l'identité projet de l'incident ou de son exécution.
