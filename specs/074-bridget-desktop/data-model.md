# Modèle de données - SPEC-074

## Profil de connexion

| Champ | Rôle | Secret | Persistance |
|---|---|---:|---:|
| `id` | identifiant stable local | non | oui |
| `label` | nom lisible choisi par l'opérateur | non | oui |
| `kind` | `ssh` | non | oui |
| `host`, `port`, `user` | cible SSH | non | oui |
| `identity_ref` | référence à une identité déjà disponible sur le Mac | non, jamais le contenu de clé | oui |
| `host_fingerprint` | identité SSH approuvée | non | oui après approbation |
| `capabilities` | UI actuelle, navigateur futur | non | oui |

Invariants :

- Un profil a un hôte, un port valide, un compte et une identité SSH gérée par le système ou explicitement référencée.
- Les anciens profils de relais direct sont écartés lors de la migration ; le client ne conserve pas leur port ni ne demande leur jeton.
- Aucun profil ne contient un mot de passe, le contenu d'une clé, un jeton UI, un cookie ou un corps de message.
- Un changement d'empreinte est une rupture de confiance et bloque la session jusqu'à décision explicite.

## Session de connexion

| Champ | Rôle |
|---|---|
| `profile_id` | origine propriétaire de la session |
| `state` | état public de connexion |
| `endpoint` | port et jeton éphémères, mémoire seulement |
| `tunnel_handle` | enfant SSH possédé par la session |
| `last_error` | erreur redacted et catégorisée |
| `retry_count` | nombre borné de tentatives visibles |

États : `disconnected`, `connecting_ssh`, `awaiting_host_approval`, `opening_tunnel`, `checking_relay`, `connected`, `reconnecting`, `failed`, `closed`.

Transitions interdites :

- `connected` ne peut pas être atteint depuis un simple processus SSH vivant : le relais doit répondre.
- Une session ne devient active qu'après la découverte SSH, l'ouverture du tunnel et la réponse HTTP authentifiée du relais.
- Une session fermée n'émet plus de notification ni de message de l'origine concernée.

## Panneau

Un panneau contient une seule `profile_id` et une seule session. Deux panneaux au plus sont ouverts simultanément dans la première version. Toute notification, titre, agent ou message rendu porte l'origine du panneau ; il n'existe pas de fusion de timeline entre serveurs.

## Capacité future navigateur

Une future capacité est déclarée par profil mais reste indisponible dans SPEC-074. Elle devra introduire une `BrowserSession` liée à une exécution précise, à un contexte isolé et à une décision explicite de reprise de main humaine. Elle ne partage ni cookies, ni écran, ni contrôle avec une autre session.

La future `BrowserSession` ouvrira un **second** canal de boucle locale, distinct du tunnel de l'UI Bridget et borné au profil sélectionné. Le navigateur restera lancé sur le serveur concerné : le Mac ne recevra qu'un flux de visualisation et des commandes explicitement autorisées. Le passage agent -> humain et humain -> agent sera une transition durable, visible et auditée ; il ne pourra pas être déduit d'un simple clic ou d'un état du runtime.
