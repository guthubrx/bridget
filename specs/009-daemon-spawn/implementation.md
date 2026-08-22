# Journal d'implémentation — équipiers gérés par le daemon

## Base de branche

La session 009 consomme le socle idempotent 012 revu. La base intégrée avant
T904/T905 est `c4010e3` (`74ad309` + correctif de verrou `1fa9dfb`).

## T905 — Ordres client, refus typés et environnement

- Le protocole expose `SpawnOrder`, `SpawnAccepted`, les onze refus fermés du
  contrat, `StopOrder` et les cinq `StopOutcome`.
- La projection CLI affiche toujours le `command_id` avant l'attente réseau.
  L'enveloppe d'un spawn est écrite en `0600` sous
  `~/.local/state/bridget/spawn-orders/` avant l'envoi ; `--command-id` relit
  exactement ces octets et refuse toute option divergente.
- La garde `forbidden_env` est partagée avec T710 et s'applique à
  l'environnement source. L'environnement enfant ne contient ensuite que la
  baseline D-505 et le `pass_env` validé du registre.
- La matrice SC-003 automatisée exerce les onze familles et vérifie qu'aucune
  réservation active supplémentaire ne subsiste après un refus.

### Gate réelle D-505 — 2026-08-22

Précondition vérifiée : `OPENAI_API_KEY`, `CODEX_API_KEY` et
`ANTHROPIC_API_KEY` absentes. Les deux adaptateurs ont été lancés sous
`env -i` avec seulement `HOME`, `PATH`, `USER`, `LANG` et `TMPDIR`; aucun
prompt n'a été envoyé.

| Type | Commande pinnée | Observable |
|---|---|---|
| Codex | `npx @zed-industries/codex-acp@0.16.0 -c model=\"gpt-5.5\"` | réponse `initialize`, protocole 1, agent `codex-acp` 0.16.0 |
| Claude | `npx @zed-industries/claude-code-acp@0.16.2` | réponse `initialize`, protocole 1, agent 0.16.2 |

Codex conserve volontairement son processus après l'EOF ; le gate l'a borné
à vingt secondes et l'a terminé après réception de la réponse `initialize`.
Claude s'est terminé proprement après la même négociation. Cette preuve valide
le lancement et la découverte de l'authentification par abonnement dans
l'environnement nettoyé, sans appel modèle ni facturation API.

### Échantillon quickstart §3

Le test ciblé de la matrice couvre notamment `UnknownType`, `CommandMissing`,
`BillingGuard`, `NameActive` et `CwdGone`, les cinq motifs proposés à
l'inspection manuelle dans le quickstart. Chaque issue est typée et le compteur
de générations actives reste inchangé par le refus.

## T906 — Canal de statut et supervision

- Le daemon libère le bootstrap seulement après le marqueur durable, puis
  supervise chaque enfant par `Child::try_wait` au tick. `BootstrapReady` ne
  produit aucune issue client ; le succès exige à la fois le `Register` réel et
  la fermeture volontaire du hook `managed-status` après initialisation du
  transport ACP, du journal et du relais attach.
- Le wrapper émet `StartupFailed` avec l'identité, la commande et la génération
  héritées du bootstrap. La disparition de la commande entre le préflight et le
  spawn est remontée comme `CommandMissing` avec le chemin exact.
- La mort spontanée reprend les terminaux 007/008 : demandes suivies rejetées,
  présence `stopped` et `End` de chaque vue attach, y compris si l'EOF socket a
  gagné la course sur le tick superviseur.
- Le stderr n'est jamais décodé comme protocole. Il reste consultable sous
  `~/.cache/bridget/managed-stderr/<nom>/<instance_id>-g<generation>/stderr.log`,
  avec répertoires `0700`, fichier `0600` dès l'ouverture. Le bootstrap, le
  wrapper et l'adaptateur ACP héritent tous de ce même descripteur ; le mode
  interactif historique conserve son stderr neutralisé. La purge applique au
  démarrage puis chaque heure la même rétention en jours que le journal du
  daemon.

Le test d'intégration exécute le vrai binaire `bridget managed-wrapper` après
le bootstrap : avant `Registered`, aucun succès n'est observable ; après la
réponse du faux daemon, l'échec réel de spawn ACP remonte un `StartupFailed`
corrélé. Un second test lance réellement un enfant supervisé qui termine avec
le code 7, puis exerce `try_wait` → événement → rejets/état/vue. Les tests de
permissions vérifient aussi les modes créés directement par `DirBuilder` et
`OpenOptions`, sans fenêtre `create` puis `chmod`.
