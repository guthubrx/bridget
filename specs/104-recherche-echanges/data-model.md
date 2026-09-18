# Modèle104 — Données existantes, curseur sans pouvoir

## Sources existantes / dépendantes

ledger existant : PK(id,target), ts, sender, target, body, conversation_key.
Une occurrence = cette clé physique : ne pas dédupliquer seulement par id.
Pas d'union avec send_deliveries : éviter double représentation, ne rechercher que le ledger.
La projection récente historique peut avoir des règles différentes, laissées intactes.

discussion_entries / discussion_members : tables prévues102, non encore en code à la base.
Clé(thread_id,seq), message_id unique, author_id, body, created_at.
Aucune table104 ; aucun état durable de recherche ni ACK. Seuls deux index ledger nouveaux.

## Objet de recherche

source=messages|thread ; query:string ; author UUID? ; peer UUID? (messages seulement) ;
since/until Unix-secondes? (bornes inclusives), limit=20 (1..50), cursor:string?.
thread_id exigé seulement pour source=thread. Aucun scope=all, project_id ni acting_agent.

Validation : query non blanche≤256octets, split_whitespace→1..8termes ; repli existant
fold_for_search ; tous les termes requis comme sous-chaînes, ordre indifférent.
Les guillemets sont littéraux, pas délimiteurs de phrase ; pas de regexp ni regex cachée.
Normalisation des accents précomposés de la table Rust existante ; compléter Ÿ→y pour
symétrie avec ÿ. Pas de normalisation Unicode NFC, translittération ou stemmer promises.
Un é accent décomposé peut ne pas correspondre à e : limite documentée et testée.
since/until≥0 et since≤until, UUID canoniques, booléens/floats refusés.

## Curseur V1

Encodage hex minuscule d'un JSON compact, taille finale≤16 384octets, parser strict.
Cette borne inclut deux tuples et l'expansion JSON/hex d'anciens identifiants de256octets ;
une borne2Kio refuserait certains identifiants pourtant admis. La sortie60Kio inclut le curseur.
Champs : v=1, actor UUID, fingerprint hexSHA256, source, upper, before.
fingerprint = SHA256 de JSON structuré à ordre fixe contenant source, query originale,
author|null, peer|null, since|null, until|null, thread_id|null. limit peut changer entre pages.
Deux textes équivalents après repli, par exemple café et cafe, gardent des fingerprints
différents : répéter la requête EXACTE pour continuer. C'est volontaire, pas un défaut
de correspondance. Pas de secret ni de texte dans le curseur (fingerprint n'est toutefois
pas une anonymisation).
actor doit être l'identité attestée. Ne pas inférer les droits depuis actor ou fingerprint.

Messages upper/before sont tuples {ts,id,target} binaires ; before est exclusive,
upper inclusive. Première upper = plus grande clé autorisée au moment de la requête
(dates comprises), pas la clé maximale de tout le ledger. Aucun curseur si corpus vide.
Même borne supérieure à toutes les pages. Tuple incomplet, tailleID/target>256, avant>upper :
invalid_cursor. ID/sender/target hérités>256octets : source_metadata_too_large, sans
résultat partiel ni curseur tronqué. Ne pas ignorer silencieusement la ligne ; l'agent
peut restreindre ses dates, mais aucune suppression/réécriture automatique n'est autorisée.
Fil upper/before sont séquences≥1 ; upper≤last_seq lu en première page.
Les bornes données par un client restent non fiables : SQL et droits s'appliquent toujours.
Modifier à la main un curseur peut relire/sauter ses propres données, jamais ouvrir celles d'autrui.
Aucun MAC/HMAC nécessaire pour ce contrat non autorisant ; ne pas prétendre antifalsification.

## Réponse search

status=ok ; source ; hits[] ; has_more bool ; next_cursor string|null ;
scanned_count (candidats AUTORISÉS traités dans cette page, même rejetés par filtres) ;
scanned_bytes (corps réellement examinés) ; skipped_oversized ;
stop_reason=exhausted|result_limit|scan_budget|byte_budget|response_budget ;
consistency=live_bounded|immutable_upper_bound ; notices[].
Les compteurs sont locaux à cette page, pas un total de l'archive ni des occurrences.
has_more=true signifie des candidats à explorer, pas forcément des occurrences à venir.
Warnings fixes de conservation/visibilité, plus restrictions concrètes ; jamais de contenu
de requête ou de tiers dans un warning.

Hit messages : kind=message, id,target,sender,ts,excerpt,match_offset,body_digest,body_bytes.
Hit thread : kind=thread_entry, thread_id,seq,message_id,author_id,ts,excerpt,
match_offset,body_digest,body_bytes.
Extrait depuis match_offset≤512octets à frontière UTF-8 ; match_offset est un offset
du corps original, body_digest son SHA256. Il localise le premier des termes trouvés,
sans garantir que les huit termes tiennent dans le même extrait.
Chaîne brute JSON sans HTML ou ANSI exécuté.
Rendu terminal neutralise les contrôles ANSI ; affichage ne change pas les données JSON.

## Relecture d'un message

Ref {id,target}, chacun chaîne non vide≤256octets (target peut être un identifiant historique
non UUID) ; offset octets≥0 frontièreUTF-8 ; digest facultatif seulement au premier appel
offset=0, obligatoire ensuite. Réponse ref, sender,ts,body_bytes,digest,fragment,next_offset|null.
Un message>16Mio renvoie source_too_large sans charger tout le corps ; aucune troncature cachée.
Digest SHA256 hex du corps exact ; comparaison après contrôle d'accès.
Source purgée/interdite : même not_found_or_forbidden ; source changée : content_changed.
La relecture ne récupère ni voisins ni conversation_key pour le client.
