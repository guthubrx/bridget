# Réemploi — PASS

- scripts/install-k1.sh : ancien ensemble Bridget + Maicie, compile et modifie
  des services. Non réutilisable pour un installateur public Bridget seul.
- scripts/deploy-remote.sh : transport SSH client, préfixe neuf, hors ligne.
  Ne répond pas à l'installation publique ni à la mise à jour sur place.
- GitHub Releases : archives natives et sidecars SHA-256 réutilisés tels quels.
- scripts/build.py : compilation et entretien du cache réutilisés pour --source.
- cartae.app : Caddy sert déjà une page /bridget.html. Un répertoire /bridget
  casserait sa résolution. Une redirection précise /bridget/install.sh vers le
  script GitHub évite ce conflit et toute copie divergente. Rechargement Caddy,
  sans redémarrage des services applicatifs.

Gate avant tasks : [x] Aucun installateur public équivalent ; pas de doublon.
