# Checklist qualité - SPEC-085

## Complétude fonctionnelle

- [x] Le défaut serveur et le choix par projet sont distingués.
- [x] L'activation Host vers Docker est spécifiée atomiquement.
- [x] Image, outils, secrets, état et rollback sont définis.
- [x] Le modèle d'un conteneur partagé par projet est explicite.
- [x] La reconnexion après redémarrage et les refus d'activité sont couverts.

## Sécurité

- [x] Aucun argument Docker, chemin ou image libre n'entre depuis l'UI.
- [x] Aucun secret n'est incorporé dans l'image.
- [x] Les limites et options de durcissement existantes sont conservées.
- [x] Aucun fallback Host silencieux n'est autorisé.
- [x] Le compte de service n'obtient aucune modification automatique de groupe ou de socket.
- [x] L'image de base reste neutre vis-à-vis des fournisseurs.

## Bornage

- [x] Aucun Kubernetes, Compose, microVM, GUI ou navigateur.
- [x] Linux amd64 est la seule architecture prise en charge dans ce lot.
- [x] Le dogfooding est reporté à SPEC-086.
- [x] Les ressources réutilisent SPEC-067.
- [x] Le moteur SPEC-066 est étendu et non remplacé.

## Verdict

PASS - La spécification peut entrer en planification sans clarification bloquante.
