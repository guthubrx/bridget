# Recherche technique — Session 038

## Décision 1 — Conserver un principal absent

**Décision** : un principal approuvé absent des marqueurs vivants est conservé
et signalé.

**Rationale** : l'absence d'un processus est une observation temporaire ; la
révocation est une décision métier. Les confondre supprimerait un droit sans
mandat et rendrait le retour de l'agent incompréhensible.

**Alternative rejetée** : retirer toute entrée absente. Cette option transforme
une collecte partielle ou un arrêt normal en révocation silencieuse.

## Décision 2 — Porter la complétude dans la politique

**Décision** : chaque principal régénérable nomme l'hôte et le chemin canonique
de sa source de marqueurs. L'outil exige exactement l'ensemble de ces sources.

**Rationale** : demander à l'opérateur de se souvenir des hôtes distants est une
règle de conduite, pas une garantie. La liste autoritative doit conduire le
contrôle, comme un match exhaustif conduit le compilateur vers les nouveaux cas.

**Alternative rejetée** : options `--source` libres sans attente stockée. Une
invocation locale seule resterait verte et plausible.

## Décision 3 — Scanner sur l'hôte qui possède les PID

**Décision** : le scanner mesure `PID + birth` localement et produit un
inventaire transportable. Pour une machine distante, il est exécuté via SSH sur
cette machine.

**Rationale** : `/proc` Linux et `proc_pidinfo` macOS n'ont de sens que sur
l'hôte observé. Rapatrier les marqueurs avant validation comparerait les PID au
mauvais noyau.

**Alternative rejetée** : intégrer un client SSH et la découverte des hôtes
dans le binaire. Cela ajoute authentification réseau, configuration et politique
de reprise sans nécessité ; le canal SSH d'exploitation existe déjà.

## Décision 4 — Hériter le grant, ne pas le recréer

**Décision** : le remplacement d'instance conserve exactement l'expiration et
la révocation du grant existant. Si plusieurs instances ne portent pas la même
sémantique, l'outil refuse.

**Rationale** : choisir ou fabriquer une expiration transformerait le
renouvellement en autorisation. La session ne possède pas cette autorité.

**Alternatives rejetées** : prolonger automatiquement l'expiration ; remettre
`revoked=false` ; choisir arbitrairement la première instance.

## Décision 5 — Prévisualisation par défaut et écriture atomique sous verrou

**Décision** : l'outil calcule sans écrire tant que `--apply` n'est pas présent.
L'application relit sous un verrou compagnon puis utilise la primitive atomique
privée existante.

**Rationale** : le fichier contient un secret et gouverne trois effets durables.
Le verrou empêche deux opérateurs de publier chacun `N+1`; le renommage évite
une politique partielle au chemin final.

**Alternative rejetée** : `fs::write` direct. Un arrêt après troncature laisserait
une politique syntaxiquement morte et refuserait toute mutation.

## Sources locales consultées

- `crates/bridget-transport/src/greffe_authorization.rs` : format, garde,
  lecture privée et invariants du grant.
- `crates/bridget-daemon/src/mcp_identity.rs` : marqueur typé et résolution du
  nom/instance.
- `crates/bridget-daemon/src/managed_process.rs` : mesure de naissance Linux et
  macOS.
- `crates/bridget-transport/src/fsutil.rs` : écriture privée atomique durable.

Aucune recherche Web n'est nécessaire : le défaut, les formats et les
primitives sont tous mesurés dans le dépôt et la politique réelle reste secrète.
