# Revue sécurité runtime Docker - SPEC-066

Date: 2026-08-30
Périmètre: revue statique ciblée et test integration runtime. Aucun déploiement,
aucune activation de projet réel et aucune modification de service.

## Vérifications positives

- Les arguments Docker sont construits par liste de paramètres sans shell.
- Le conteneur reçoit une racine lecture seule, suppression de toutes les
  capabilities et option no-new-privileges.
- Identité conteneur: UID et GID non nuls, contrôlés contre identité hôte avant
  préparation. Le test réel confirme uid et gid attendus.
- Réseau: bridge sans publication de port et sans réseau hôte.
- Montages: validation refuse racine hôte, cache Bridget et socket Docker. Le
  résolveur admet seulement racine projet, worktrees Git du même common dir,
  state root projet et ingress privé.
- Etat durable: seul state root est inscriptible; HOME et XDG restent sous ce
  répertoire. Le test réel vérifie écriture state root et lecture fixture 0600.
- Ingress: socket déterministe, répertoire 0700 et socket 0600. Une connexion
  sans réservation est refusée avant inscription.
- Exécution: une policy peut déclarer seulement un lanceur et une commande
  internes, absolus et sans arguments libres. Les chemins du registre hôte ne
  sont pas utilisables comme commande conteneur.
- Pas de fallback: SpawnOrder Docker est refusé avant création de processus hôte.

## Limites encore ouvertes

- Egress bridge volontairement non filtré finement en v1.
- Handshake ingress accepté, réservation et docker exec ne sont pas encore
  réalisés. Le socket refuse donc tout client non réservé.
- Aucun secret ni credential ne sont injectés par SPEC-066. Le catalogue,
  secret source et redaction restent de la responsabilité de SPEC-067.
- Les contrôles OOM, PID limit, exec lost et réconciliation après Docker restart
  restent à implémenter et tester.

## Verdict

PASS pour environnement fixture US1, frontières de montage et refus explicite
de repli host. Non approbateur pour lancement agent Docker et cycle de vie US2
et US3 tant que les tâches ouvertes ne sont pas prouvées.
