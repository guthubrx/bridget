# Proposition de signature des publications - lot 15

Date : 10 novembre 2026. Statut : proposition. Décision D8 : demandée à François. Rien n'est mis en place : aucune clé, aucun outil installé, aucun script modifié.

## 1. Constat (lu dans le dépôt)

- L'installateur public `scripts/install.sh` télécharge l'archive et un fichier d'empreinte voisin depuis la **même release** : `releases/download/$version/$archive.sha256` puis `.../$archive` (lignes 413 et 418). Il compare ensuite le SHA-256 calculé à l'empreinte lue (ligne 419 : « SHA-256 mismatch. Installed Bridget was not changed. »).
- Il valide la forme de l'étiquette de version (`^v[0-9]+\.[0-9]+\.[0-9]+...`), la longueur et le contenu hexadécimal de l'empreinte, et il extrait un seul membre connu de l'archive (jamais de chemins arbitraires).
- Le contrat de l'installateur est testé hors réseau par `scripts/tests/test_install.py` (utilisé aussi par le contrôle distant `repository-hygiene`).
- Procédure de publication : **non lue**. La mémoire de la séance cite un script `publier.py`, absent du dépôt public à la racine de `scripts/`. Qui signe, où, avec quelle clé : inconnu.

## 2. Menace

Si l'emplacement de publication est compromis, l'archive et son empreinte peuvent être remplacées ensemble. L'installateur ne le détecte pas, car l'empreinte vient de la même source que l'archive.

## 3. Options

| Option | Qui garde la clé | Ce que vérifie l'utilisateur | Effort | Limites |
|--------|------------------|------------------------------|--------|---------|
| A : ne rien changer | personne | l'empreinte publiée au même endroit | nul | la menace ci-dessus reste entière |
| B : signature détachée (`minisign`, `gpg` ou `ssh-keygen -Y sign`, outil à choisir) | un détenteur de clé privée hors ligne, à désigner par François | une signature de l'archive avec une clé publique intégrée à l'installateur | moyen : outil de signature, gestion de la clé, changement de l'installateur et de ses tests | perte ou fuite de la clé : à traiter (rotation) ; l'installateur doit embarquer la clé publique ; l'outil de vérification doit exister chez l'utilisateur (à vérifier par plateforme) |
| C : attestations de provenance de la plateforme d'hébergement | aucune clé à garder (liée au flux de publication) | une attestation liée à la source et au flux de publication | à vérifier : dépend de la façon dont les archives sont construites et publiées aujourd'hui (non lue) | l'utilisateur a besoin d'un outil de vérification de la plateforme ; la disponibilité sur le dépôt et le flux actuels est à vérifier |

## 4. Effet sur l'installateur (sans l'écrire)

Pour B ou C, le bloc de téléchargement et de comparaison d'empreinte (autour des lignes 413 à 419 de `scripts/install.sh`) gagnerait une étape de vérification de signature avant `tar`. La fonction `download` serait appelée pour un fichier de plus. Les tests `scripts/tests/test_install.py` devraient couvrir : signature valide, signature absente, signature invalide, clé inconnue.

## 5. Décision demandée à François

Choisir A, B ou C. Pour B : désigner qui garde la clé privée et choisir l'outil. Aucun délai n'est imposé par ce lot.

## 6. Ce qui n'est pas fait

Aucune clé générée, aucun outil installé, aucun script de signature écrit, aucun changement de `scripts/install.sh`. La procédure de publication réelle n'a pas été lue.
