# Correction202 — Reprise après choix d’effort

Un Start refusé inbound_queue peut être repris sur la même tâche. Un choix high pour un profil low possède une empreinte de définition différente. La reprise doit comparer la définition d’exécution admise et conserver la sélection durable ; les contrôles worker avant préparation, Start et Release restent stricts. Toute dérive réelle rend un diagnostic fermé précis, sans mutation du reçu ni nouvelle tâche. Incident92965c7d, cléopus167-cache-candidats-sans-copie-20261028-01.
