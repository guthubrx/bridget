# Recherche technique - SPEC-074

## Faits observés dans le dépôt

| Sujet | Observation | Conséquence |
|---|---|---|
| Relais UI | `bridget-ui.service` sert actuellement `127.0.0.1:17888` et refuse toute adresse non loopback. | Préserver cette limite ; le client transporte l'accès avec SSH. |
| Endpoint UI | `UiEndpoint` conserve port et jeton dans `/home/moi/.cache/bridget/ui-endpoint.json`, droits 0600. | Ne pas lire ce fichier depuis Desktop ; exposer une découverte CLI étroite. |
| Tunnel existant | `scripts/open-remote-ui.sh` emploie déjà SSH `-L` pour relier une boucle locale cliente à la boucle locale distante. | Réutiliser le principe, pas le script interactif ni son deuxième relais temporaire. |
| UI actuelle | Le daemon embarque les assets et fournit les API HTTP/SSE. | Bridget Desktop rend l'UI déjà servie, sans duplication initiale. |
| Workspace | Aucun client desktop ni `package.json` Tauri n'existe dans le workspace. | `apps/bridget-desktop` est un nouveau périmètre clairement séparé. |
| SPEC-002 | La fédération inverse utilise SSH, `ExitOnForwardFailure` et des keepalives. | Reprendre ces garanties opératoires côté tunnel client, sans modifier la fédération des agents. |

## Sources externes vérifiées le 2026-08-30

- Tauri isole les permissions par capability, fenêtre et webview. Les sources distantes ne doivent pas recevoir de privilèges locaux par défaut.
  https://tauri.app/security/capabilities/
  https://v2.tauri.app/reference/acl/capability/
- Tauri documente des scopes spécifiques aux commandes et plugins. Un shell trop large exposerait l'application locale au contenu distant ; le backend doit donc lancer uniquement SSH avec des arguments construits et validés.
  https://v2.tauri.app/security/scope/
  https://v2.tauri.app/zh-cn/plugin/shell/
- Tauri 2 expose des webviews enfants avec URL externe et un contrôle de navigation. Cette API nécessite aujourd'hui le feature Cargo `unstable`; c'est une dette explicitement bornée par le besoin de deux panneaux dans une même fenêtre, et chaque webview peut être ciblée séparément dans les capabilities.
  https://docs.rs/tauri/latest/tauri/webview/struct.WebviewBuilder.html
  https://v2.tauri.app/reference/acl/capability/
- OWASP rappelle que les clés SSH et les jetons sont des secrets et que les journaux doivent empêcher leur fuite tout en conservant une valeur opérationnelle.
  https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html
- NIST SSDF demande de protéger les composants contre l'altération, documenter les exigences et risques, puis traiter les vulnérabilités de livraison.
  https://csrc.nist.gov/projects/ssdf
- W3C demande que les formulaires et la navigation soient entièrement utilisables au clavier, avec un ordre de focus logique et sans piège.
  https://www.w3.org/WAI/fundamentals/accessibility-principles/
  https://www.w3.org/WAI/WCAG22/Understanding/focus-order
- Playwright fournit des contextes navigateur isolés et non persistants. Cette propriété est retenue uniquement comme exigence de la capacité navigateur future.
  https://playwright.dev/docs/next/browser-contexts
- noVNC est un client VNC HTML5 utilisant WebSocket et Canvas. Il est une option future possible pour visualiser un navigateur serveur via un tunnel SSH, non une dépendance de cette SPEC.
  https://novnc.com/info.html

## Décisions de recherche

1. Le tunnel est possédé par Bridget Desktop, côté Mac. Le serveur ne reçoit aucune nouvelle écoute publique.
2. Le client SSH système est préféré à une nouvelle bibliothèque cryptographique : il réutilise l'agent, les clés, les hôtes connus et les politiques déjà opérées sur le Mac.
3. La configuration de profil reste non secrète. Une clé est une référence à l'identité macOS, jamais une valeur importée par défaut.
4. La page distante est traitée comme du contenu non privilégié même si elle arrive via SSH. La coque locale conserve seule les capacités système.
5. La double vue est limitée à deux panneaux pour préserver lisibilité et ressources ; elle ne crée pas d'agrégat métier inter-serveurs.
6. Le navigateur distant n'est pas anticipé par une abstraction de tunnel générique. Sa future SPEC devra prouver isolement de contexte, durée de vie, reprise humaine et flux vidéo.
7. Les deux panneaux utilisent des webviews enfants externes marquées `panel-*`. Le feature Tauri `unstable` est retenu uniquement ici, afin que leurs labels ne reçoivent aucune capability de la coque locale; son évolution est un risque de mise à jour à surveiller.

## Risques retenus

- Une empreinte SSH acceptée silencieusement permettrait une substitution de serveur. L'acceptation initiale et le changement sont donc des états explicites et bloquants.
- Un jeton dans une URL est nécessaire au contrat UI actuel mais ne doit pas atteindre logs, stockage de profil ou diagnostic. Le contrat de découverte sera testé sur ce point.
- Un contenu servi par un serveur compromis ne doit jamais obtenir le droit de lancer SSH, lire une clé ou accéder à l'application locale.
- Deux panneaux peuvent dégrader l'expérience ou mélanger les états si leur origine n'est pas portée par chaque action et notification.
- Le support multiwebview Tauri porte un feature `unstable`. Un changement de son API peut réclamer une adaptation de Bridget Desktop lors d'une mise à jour Tauri, mais n'affecte ni le protocole Bridget ni les profils persistés.
- Le binaire macOS ne peut pas être produit sur le serveur Linux. Cette validation cible est un gate de sortie, non une prétendue preuve Linux.

## Réutilisation imposée

- Le relais UI actuel et ses contrats HTTP/SSE.
- `UiEndpoint` et sa persistance uniquement à travers une commande dédiée.
- Le principe de robustesse SSH de la SPEC-002.
- Les états honnêtes de remise, activité, erreur et notification des SPEC-063 à SPEC-070.
- Le registre d'identité runtime et fournisseur des SPEC-071 et SPEC-072.

## Exclusions confirmées

- Aucun serveur Web nouveau, tunnel TCP générique, VPN, proxy public ou gestionnaire de clés privé maison.
- Aucun navigateur, VNC, noVNC, Playwright ou dépendance associée dans cette SPEC.
