# Réseau mobile 177

DNS A du sous-domaine réel vérifié vers Cartae. Caddy réutilisé : autres sites préservés, configuration validée puis reload réussi. Certificat TLS public valide. Backend r6 limité à 127.0.0.1:9467 ; requête HTTPS réelle sans clé vers /v1/directory : HTTP 401, vérification TLS 0. Proxy borné à 12 Mo et délais finis, adresse source remplacée par le proxy de confiance. Aucun port TCP du protocole natif n’est publié. Les preuves brutes de configuration et de DNS restent privées.

La liaison des clients Mac utilise une socket Unix 0600 et SSH via socat, direction to-master. L’ancien reverse vers le Mac est retiré avec l’installateur natif. Aucun paramètre SSHD ni port autorisé n’est élargi.
