# Couverture225 — option de présentation globale

Le dispatcher retire --table avant les parseurs métier. Son emplacement avant la
commande ou après celle-ci ne change ni droits, portée, reçus ou codes de retour.
Le renderer reçoit la sortie locale de cette unique exécution. Il ne relance
jamais Bridget et n'appelle aucun fournisseur. Aucun protocole MCP n'est modifié.

| Familles et opérations | Raccord et colonnes |
| --- | --- |
| agents, who, discover | Même annuaire natif ; nom humain, domaine/type, projet, hôte, état. |
| specialists list et request | Nom humain, rôles, portée projet, hôte, état ; profils et mémoire : colonnes propres. Confirmation humaine intacte. |
| task list/status/wait/ack/cancel/continue/respond/cleanup ; control delegations | Référence, agent, placement, état, ronde. Fiche détaillée avec causes, gardes et actions. |
| control status/budget/posture/pause/resume/halt-delegations/resume-delegations | État et reçus indicateur/valeur. Budget issu de ControlStateFrame, aucune consommation estimée. |
| requests, inbox, queue | Référence/position, origine/destination ou type, état, choix/contexte. |
| ledger, ledger search/read | Page bornée native : référence, correspondants, texte, date/état. Curseur et indicateur de suite conservés. |
| journal, events | Séquence, type, agent, résumé, date. Détails et lacunes natifs conservés. |
| wake list/cancel/schedule, thread create/list/read/post/ack/... ; handoff preview/send | Cible/référence, titre/texte, responsable/projet, état/date ; métadonnées et reçus. |
| thread inspect, lineage inspect/cancel sans --follow | Projection humaine bornée. JSON ajouté uniquement au parseur qui l'exige. |
| artifact hors read/publish | Métadonnées : référence, nom, type, taille, projet/date. |
| delegate --catalogue [--observed/--refresh-models], delegate, auto-delegate | Machine, connexion, moteur, modèle, efforts, observation ; reçus distincts du catalogue. |
| accounts list/prepare/enable/disable/remove | Connexion, moteur, machine, activation, auth non vérifiée, gestion. Aucun secret ajouté. |
| updates status/check/auto/install/prepare/finish/report/grant | Réponse native224 : hôte, date observée, version installée/disponible, état/action. Installé ne signifie pas chargé. |
| mobile status/devices et administration finie | Appareil, état, portées, version/date ou reçu ; garde humaine conservée. |
| clients check/policy/validate-report ; formats policy/report/validate-report | Contrôles/versions/verdicts/actions ou clé/valeur. Aucun contrôle physique ajouté. |
| federate status | Une exécution du helper embarqué ; état de processus en indicateur/valeur. Connectivité jamais déduite. |
| worker diagnose ; identity ; runtime ; domain ; dnd ; reaper report ; reprise | Diagnostics, paramètres et cartes bornés en clé/valeur. Écritures existantes inchangées. |
| spawn, stop, relaunch, decommission, adopt-stopped, send, reply, cancel, rename, guichet, delegate-grant | Reçus finis clé/valeur, sans action/clé supplémentaire ; garde TTY intacte. |

## Exclus avant toute opération avec --table

- daemon, worker hors diagnose, mcp, t3, managed-bootstrap/managed-wrapper,
  observateur interne, hooks : transports ou services.
- Wrappers fournisseur et arguments après -- : pas d'interception.
- attach, mobile pair/serve, updates setup/watch/once, thread watch,
  lineage watch, --follow : interaction, appairage ou flux continu.
- artifact read/publish, helpers workspace-view-* : contenu brut ou amorçage.
- version/help/install-hooks : aide, scalaire ou installation hors inventaire.

Champ absent : « — » ou « Non observé ». Schéma inconnu : clé/valeur.
Pas de remplissage automatique d'une page ; curseur visible.
Pas de couleur sans TTY, avec NO_COLOR ou TERM=dumb.
Largeur OS puis COLUMNS puis120 ; cellules repliées, sans troncature.
Buffer16Mio ; au-delà, sortie originale et avertissement, sans rejeu.

## Revue de conception

e32e917c / Haiku SYS : inventaire lu, résultat acquis/ACK, clôture demandée.
Retenu : parseur avant effets, valeurs d'option protégées, presence réelle,
Unicode/largeur, curseurs, garde TTY et MCP séparé.
La réduction proposée à cinq familles n'est pas retenue : l'utilisateur demande
toutes les sorties pertinentes. Les autres parseurs sont examinés par le
responsable. Présenter la sortie CLI de queue ne modifie pas son protocole MCP.

Pour send/reply, placer --table avant le premier mot du corps, ou avant la commande. Un texte après -- reste intact. who/discover gardent leurs gardes, pieds de contrôle et avertissements natifs.
