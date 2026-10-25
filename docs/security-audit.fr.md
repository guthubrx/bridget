# Vérification de confidentialité — 23 octobre 2026

[English](security-audit.md) · [Français](security-audit.fr.md) · [Sécurité](../SECURITY.fr.md)

## Résultat

**Aucune vraie clé d’API ni clé privée n’a été identifiée par les contrôles réalisés.**
Ce résultat a des limites ; il ne garantit pas l’absence de toute donnée sensible.

Gitleaks 8.30.1 a signalé 82 occurrences historiques, examinées dans leur contexte :
empreintes de fichiers/builds, identifiants Git, valeurs fictives de tests,
identifiants de demandes ou nonce WebSocket standard. Le fichier
`.gitleaksignore` conserve leurs seules empreintes exactes, liées au commit,
au chemin, à la règle et à la ligne. Aucun dossier ni règle entière n’est exclu.

Des données d’exploitation réelles étaient présentes : deux IP publiques,
un nom d’hôte SSH et une adresse personnelle. Les fichiers courants emploient
désormais des exemples. Deux liens absolus `node_modules` vers un poste de
travail ont été retirés. Les noms génériques de compte/machine n’ont pas été
traités comme des secrets.

## Périmètre

Base : `320283ef770b6d3a58569c62b02c6153145df722`.

- Dépôt complet, toutes les références locales disponibles : 2 302 commits
  accessibles au premier inventaire, 2 301 dans origin/main.
  Le scanner a traité 2 000 commits avec diff ; ce compteur diffère des merges.
- Règles standard Gitleaks, contrôle des contenus décodés et archives jusqu’à deux niveaux.
- Inventaire de 3 500 chemins suivis : noms sensibles, clés privées, URL
  authentifiées, courriels, IP publiques et liens absolus.
- Les quatre URL avec identifiants étaient des tests négatifs fictifs.
- OCR local de 52 PNG courants, puis scan sans alerte ; examen visuel d’une
  capture native représentative. Les anciennes images n’ont pas été OCRisées.
- Deux archives compressées incluses dans le passage adapté ; contenu de
  l’archive de livraison inventorié.
- Aucun identifiant fournisseur n’a été testé auprès d’un service externe.
- Aucune utilisation de clé suspecte, rotation, modification de compte ou relance.

Le scanner provient de sa release GitHub officielle ; son archive a été
vérifiée par SHA-256. Le workflow fige cette empreinte et le commit de l’action checkout.

## Prévention

Les guides anglais et français utilisent des adresses d’exemple. Les archives
restent dans leur langue ; seules les valeurs d’exploitation identifiées sont
assainies. Une ancienne empreinte de manifeste décrit donc l’artefact original,
pas les documents assainis.

Le `.gitignore` couvre davantage de fichiers de secrets/état et de dossiers de
dépendances. Il ne retire pas les fichiers déjà suivis. Un contrôle CI
vérifie les secrets, les liens des pages publiques et les paires de langues.

Avec Gitleaks 8.30.1 installé, depuis une copie Git complète :

```sh
bash scripts/check-secrets.sh
python3 scripts/check-docs.py
```

Le premier contrôle examine l’historique, pas les modifications non commitées.
Contrôle supplémentaire de l’index avant commit :

```sh
gitleaks git . --staged --redact=100 --no-banner --ignore-gitleaks-allow
```

## Limites et exposition restante

**Les anciennes valeurs restent dans l’historique Git.** Aucun historique public
n’a été réécrit et l’identité des auteurs reste intacte. Une suppression de
toutes les références publiées exige une opération coordonnée distincte.
Aucune clé active trouvée ne justifie une rotation immédiate sur la base de ce contrôle.

Les politiques internes, specs, notes et rapports restent suivis et lisibles
si le dépôt est public. Les séparer dans l’index ne les rend pas privés.

Les issues, discussions, pièces jointes de releases, forks/clones externes,
références non récupérées, comptes fournisseur et fichiers locaux ignorés ne
sont pas audités. L’OCR peut manquer du texte ; les formats de clés inconnus et
les données sensibles en prose peuvent échapper aux règles.

Ce rapport ne constitue ni une purge de l’historique ni une certification de sécurité.

La date de ce rapport utilise le fuseau Europe/Paris. Les adresses locales de
boucle et les valeurs d’exemple restantes ne sont pas des données d’exploitation.

CODEOWNERS demande la revue du mainteneur pour les scripts, exclusions et workflow.
Son caractère obligatoire dépend des règles de branche GitHub, non modifiées ici.
La CI utilise les fichiers de la révision proposée : elle ne protège pas contre
une modification malveillante de ses propres contrôles sans revue de confiance.

## Valeur de notification de test historique — 25octobre2026

Le contrôle de livraison sur tout l’historique a trouvé un faux positif
supplémentaire dans le commit `6f2ad5166ec1762b6726fbbf51acaf3ebc79c73f`,
fichier `crates/bridget-daemon/src/mobile/store.rs`, ligne946, règle
`generic-api-key`. Il s’agit du destinataire de notification du test unitaire
`mobile177_notification_reassociation_moves_receiver_without_moving_authority`.
Ce test utilise `Store::memory()`, des dates fictives et des clés locales de
jumelage générées. Il vérifie le changement de destinataire sans modifier les
droits de l’appareil. Il ne contacte aucun service de notification externe.
La chaîne contient deux mots en minuscules séparés par des tirets bas et la
suite123456789. C’est une donnée de test, pas un identifiant fournisseur. Sa
valeur n’est pas recopiée ici. L’exception porte uniquement sur l’empreinte
immuable commit/chemin/règle/ligne. Aucun chemin courant, nouveau commit ou
règle du scanner n’est exclu.
