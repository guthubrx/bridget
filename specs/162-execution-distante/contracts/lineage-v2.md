# Contrat de lecture Lineage 162

## Compatibilité et autorisation

Le reader T3 actuel décode v1 avec onExcessProperty:error. Garder la sortie v1 par défaut strictement inchangée, y compris pour une tâche distante. Le nouveau client demande `bridget lineage inspect --response-version 2` uniquement pour list/show. Étendre la requête existante dans lineage_protocol.rs, pas créer un type parallèle. La version de projection 2 ne change pas l’authentification HumanLineageViewV1 ni les gardes parent/projet. Un succès list/show enrichi porte version=2. Les erreurs conservent version=1 et leurs codes historiques ; le nouveau CLI accepte explicitement cette enveloppe pour une tentative v2. Un succès portant une autre version est refusé. Ajouter lineage_client.rs aux surfaces modifiées, car il exige actuellement version=1.

Le nouveau reader tente v2. Une seule lecture v1 sans ce flag est permise sur refus structuré unsupported_version ou invalid_request attribué à cette tentative. Le CLI 160 retourne invalid_request JSON, RC2, pour un flag inconnu. Timeout, refus d’autorisation, données malformées ou autre code : conserver l’erreur, sans repli général. Aucun nouvel enum fermé envoyé à un ancien serveur avant compatibilité établie.

## Snapshot

V2 conserve génération, séquence, curseurs, propriété et limite de 4 096 tâches. Une tâche reçoit placement nullable : machine_id, nom configuré facultatif, worker_id, last_contact_at et contact_state. Les champs fournisseur/modèle existants restent la référence. last_contact est observé au maître depuis le transport. L’interface affiche cette date comme observation lors de la lecture de la page, jamais comme âge vivant non rafraîchi. Il reste volatile ; seules les transitions de contact_state provoquent notification. Les transitions de contact_state incrémentent seq et invalident le curseur. last_contact_at reste une observation lors de la lecture de chaque page, sans gel global. Le snapshot métier conserve ses garanties existantes.

Si une page v2 exige le repli autorisé, abandonner tout le staging v2 et reprendre v1 depuis le curseur nul. Ne jamais mélanger versions, pages ou générations. En v1 de repli, afficher « Métadonnées distantes indisponibles », jamais une machine locale supposée.

## Réemploi et présentation

Conserver les quatre RPC read/watch/journal/cancel et leurs gardes orchestration:read/operate. Watch v1 reste inchangé. Aucun repli de version pour journal ou cancel. Garder la projection thread:bridget-task:<uuid> sans session fournisseur, absente de la colonne des conversations. Afficher machine et contact dans le Lineage, l’infobulle et le détail existants.

bridgetLineage.available décrit le maître ; contact_state décrit le worker. Le journal durable est distant et lu à la demande par le maître. S’il est indisponible, l’afficher explicitement. Le résultat et le dernier état déjà reçus restent au maître. Aucun miroir de journal ni store T3 supplémentaire.

## Oracles

Succès v2 portant une version incorrecte refusé ; erreurs v1 historiques reconnues ; fixture v1 gelée et ancien décodeur strict sur la sortie par défaut ; v2 list/show ; ancien CLI invalid_request ; ancien maître unsupported_version ; refus d’autorisation sans repli ; page 2 refusée et abandon total du staging ; transition de contact invalidant le curseur, sans incrément de séquence à chaque heartbeat ; maître accessible et worker perdu ; fermeture/reprise sans lancement fournisseur ; annulation autorisée native. Tests ciblés T3 seulement. Aucune relance de l’application ou du serveur T3. Un rendu isolé ne prouve pas le rendu actuellement chargé.
