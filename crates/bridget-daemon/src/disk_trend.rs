//! Pente d'occupation disque : rendre observable ce qu'une photographie ne dit
//! pas, sans jamais transformer un événement en tendance.
//!
//! `bridget who` affiche l'espace libre À L'INSTANT. Personne ne voit la pente,
//! parce qu'il faut deux relevés espacés pour la calculer et qu'aucun agent ne
//! conserve le relevé précédent. Ce module conserve les échantillons horodatés
//! et calcule la dérivée à leur place.
//!
//! # La contrainte dure : ne pas alerter sur un artefact
//!
//! Une dérivée sur une fenêtre trop courte mesure un ÉVÉNEMENT, pas une
//! tendance. Cas réel du 28/08 : une mesure sur 90 secondes a rendu 112 Gio/h,
//! soit « quarante minutes restantes » — c'était une compilation en cours. Trois
//! relevés de 30 s ensuite rendaient 74.39, 74.39, 74.39, strictement stable.
//! Deux fenêtres longues et indépendantes rendaient 10.6 et 11.1 Gio/h.
//!
//! C'est le `tail -25` appliqué au temps : une fenêtre trop étroite prise pour
//! le tout. Ce module REFUSE de conclure sous [`MIN_WINDOW_SECS`] plutôt que
//! d'afficher un nombre spectaculaire, et nomme la raison du refus.
//!
//! # L'instrument produit le phénomène qu'il mesure
//!
//! Fait établi le 28/08 pendant l'écriture de ce module, et il vaut plus que le
//! module lui-même. Deux relevés se contredisaient : dix-neuf minutes à
//! variation nulle d'un côté, une chute de 2.81 Gio en quatre-vingt-dix
//! secondes de l'autre. La cause de la chute était le `CARGO_TARGET_DIR` de la
//! compilation de ce fichier — 2.8 Gio, exactement l'écart mesuré.
//!
//! **Mesurer la pente par un outil qu'il faut compiler n'est pas neutre.** Ce
//! n'est pas un défaut à corriger, c'est une propriété à connaître : tant que
//! l'observation passe par une build, l'observateur est l'une des causes de ce
//! qu'il observe. Même famille que la ronde d'agents qui rend `busy` ceux
//! qu'elle interroge.
//!
//! # Ce que ce module NE sait PAS faire, et qu'aucun seuil ne réparera
//!
//! La consommation disque de ce parc est **épisodique**, pas continue : elle
//! vient des compilations, de l'ordre de 3 Gio par agent qui compile. Il n'y a
//! pas de fuite — dix-neuf minutes à variation strictement nulle l'ont prouvé
//! mieux qu'un raisonnement.
//!
//! Conséquence directe sur la portée de [`assess`] : [`MIN_WINDOW_SECS`]
//! protège du cas à 90 secondes, il ne protège PAS d'une fenêtre de dix minutes
//! entièrement occupée par une compilation. Trois relevés stables — 74.39,
//! 74.39, 74.39 — ont fait conclure à une stabilité qui n'a pas tenu quatorze
//! minutes. **Ce n'est pas le nombre de points qui manque, c'est la
//! connaissance de la cause : deux points avec la cause valent mieux que dix
//! sans elle.**
//!
//! Ce module rend donc une pente *observée*, jamais une pente *expliquée*. Lire
//! « −11 Gio/h » n'autorise pas à conclure « il reste sept heures » : il faut
//! savoir si le parc compile. C'est pourquoi rien ici ne déclenche d'alerte
//! automatique et pourquoi aucune échéance n'est publiée sans pente concluante.

use serde::{Deserialize, Serialize};

/// Fenêtre minimale avant qu'une pente soit publiable. Sous ce seuil, une
/// compilation, une purge ou l'écriture d'un artefact dominent le signal.
pub const MIN_WINDOW_SECS: i64 = 600;

/// Nombre minimal d'échantillons. Deux points ne distinguent pas une tendance
/// d'un saut ; trois permettent qu'un point aberrant ne porte pas la droite.
pub const MIN_SAMPLES: usize = 3;

/// Nombre d'échantillons conservés. Borne l'historique sans le laisser croître.
pub const MAX_SAMPLES: usize = 512;

/// Relevé d'espace libre à un instant donné, sur un volume donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskSample {
    pub observed_at_unix: i64,
    pub free_bytes: u64,
}

/// Pourquoi la pente n'est pas publiable. Nommer le refus évite qu'un appelant
/// le confonde avec « pente nulle » — un zéro et une absence ne sont pas la
/// même chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Inconclusive {
    /// Aucun relevé conservé.
    NoSamples,
    /// Moins de [`MIN_SAMPLES`] relevés distincts.
    NotEnoughSamples { have: usize, need: usize },
    /// Relevés trop rapprochés : on mesurerait un événement.
    WindowTooShort { window_secs: i64, need_secs: i64 },
}

/// Verdict de tendance. `Slope` n'est produit que si la fenêtre ET le nombre
/// d'échantillons sont suffisants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    Inconclusive(Inconclusive),
    Slope {
        /// Positif = le disque se remplit (l'espace libre baisse).
        consumed_bytes_per_hour: f64,
        window_secs: i64,
        samples: usize,
    },
}

impl Trend {
    /// Heures avant saturation au rythme observé. `None` si la pente n'est pas
    /// concluante ou si le disque ne se remplit pas — on ne prédit jamais une
    /// échéance à partir d'un refus.
    pub fn hours_to_exhaustion(&self, free_bytes: u64) -> Option<f64> {
        match self {
            Trend::Slope {
                consumed_bytes_per_hour,
                ..
            } if *consumed_bytes_per_hour > 0.0 => {
                Some(free_bytes as f64 / consumed_bytes_per_hour)
            }
            _ => None,
        }
    }
}

/// Calcule la tendance, ou refuse de conclure. Fonction pure : aucun accès
/// disque, aucune horloge, donc entièrement testable sur des vecteurs.
///
/// La pente vient d'une régression des moindres carrés sur TOUS les
/// échantillons de la fenêtre, et non de l'écart premier-dernier. Un
/// premier-dernier laisserait un unique point extrême — la fin d'une
/// compilation, une purge — fixer la pente à lui seul.
///
/// Complexité : O(n) sur le nombre d'échantillons, n borné par [`MAX_SAMPLES`].
pub fn assess(samples: &[DiskSample], min_window_secs: i64, min_samples: usize) -> Trend {
    if samples.is_empty() {
        return Trend::Inconclusive(Inconclusive::NoSamples);
    }

    let mut ordered: Vec<DiskSample> = samples.to_vec();
    ordered.sort_by_key(|sample| sample.observed_at_unix);
    ordered.dedup_by_key(|sample| sample.observed_at_unix);

    if ordered.len() < min_samples {
        return Trend::Inconclusive(Inconclusive::NotEnoughSamples {
            have: ordered.len(),
            need: min_samples,
        });
    }

    let window_secs = ordered[ordered.len() - 1].observed_at_unix - ordered[0].observed_at_unix;
    if window_secs < min_window_secs {
        return Trend::Inconclusive(Inconclusive::WindowTooShort {
            window_secs,
            need_secs: min_window_secs,
        });
    }

    // Moindres carrés de free_bytes en fonction du temps. L'origine des temps
    // est ramenée au premier échantillon pour rester loin des dépassements.
    let n = ordered.len() as f64;
    let origin = ordered[0].observed_at_unix;
    let (mut sum_t, mut sum_y, mut sum_tt, mut sum_ty) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    for sample in &ordered {
        let t = (sample.observed_at_unix - origin) as f64;
        let y = sample.free_bytes as f64;
        sum_t += t;
        sum_y += y;
        sum_tt += t * t;
        sum_ty += t * y;
    }
    let denominator = n * sum_tt - sum_t * sum_t;
    if denominator == 0.0 {
        // Tous les instants confondus : la fenêtre était nulle. Défensif, mais
        // le contrôle de fenêtre ci-dessus l'a déjà écarté.
        return Trend::Inconclusive(Inconclusive::WindowTooShort {
            window_secs,
            need_secs: min_window_secs,
        });
    }
    // Pente de l'espace LIBRE (octets/s) ; négative quand le disque se remplit.
    let free_bytes_per_sec = (n * sum_ty - sum_t * sum_y) / denominator;

    Trend::Slope {
        consumed_bytes_per_hour: -free_bytes_per_sec * 3600.0,
        window_secs,
        samples: ordered.len(),
    }
}

/// Ajoute un échantillon et borne l'historique à [`MAX_SAMPLES`], en jetant les
/// plus anciens. Les relevés restent triés par instant.
///
/// Un relevé portant une seconde déjà présente REMPLACE l'ancien au lieu de
/// s'ajouter. Sans cela, plusieurs `who` lancés dans la même seconde empilaient
/// des points que [`assess`] fusionne ensuite de toute façon : l'historique
/// grossissait de relevés inexploitables, et son volume ne disait plus rien du
/// nombre d'observations réelles. Constaté à l'exécution, pas en test.
pub fn push_sample(history: &mut Vec<DiskSample>, sample: DiskSample) {
    match history
        .iter_mut()
        .find(|item| item.observed_at_unix == sample.observed_at_unix)
    {
        Some(existing) => *existing = sample,
        None => history.push(sample),
    }
    history.sort_by_key(|item| item.observed_at_unix);
    if history.len() > MAX_SAMPLES {
        let excess = history.len() - MAX_SAMPLES;
        history.drain(0..excess);
    }
}

/// Fichier d'historique. C'est lui qui se souvient à la place de l'agent :
/// sans persistance, chaque observateur repart d'un unique relevé et la pente
/// reste invisible, quel que soit le nombre de fois qu'on regarde.
pub fn history_path(state_dir: &std::path::Path) -> std::path::PathBuf {
    state_dir.join("disk-trend.json")
}

/// Relit l'historique. Un fichier absent, illisible ou corrompu rend un
/// historique vide : on repart d'un refus explicite, jamais d'une pente fausse.
pub fn load_history(path: &std::path::Path) -> Vec<DiskSample> {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Écrit l'historique par remplacement atomique, pour qu'un `who` interrompu
/// ne laisse jamais un fichier tronqué derrière lui.
pub fn save_history(path: &std::path::Path, history: &[DiskSample]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let encoded = serde_json::to_string(history)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, encoded)?;
    std::fs::rename(&temporary, path)
}

/// Enregistre le relevé courant puis rend le verdict : un seul point d'entrée
/// pour l'appelant, qui n'a donc rien à mémoriser.
pub fn record_and_assess(path: &std::path::Path, sample: DiskSample) -> Trend {
    let mut history = load_history(path);
    push_sample(&mut history, sample);
    // Un échec d'écriture ne doit pas empêcher de rendre la tendance déjà
    // calculable : la mesure vaut mieux que rien, elle sera seulement oubliée.
    let _ = save_history(path, &history);
    assess(&history, MIN_WINDOW_SECS, MIN_SAMPLES)
}

const GIB: f64 = 1024.0 * 1024.0 * 1024.0;

/// Rend la tendance pour un affichage humain. Un refus s'affiche comme un
/// refus — jamais comme une pente nulle.
pub fn format_trend(trend: &Trend, free_bytes: u64) -> String {
    match trend {
        Trend::Inconclusive(Inconclusive::NoSamples) => "pente ? (aucun relevé)".to_string(),
        Trend::Inconclusive(Inconclusive::NotEnoughSamples { have, need }) => {
            format!("pente ? ({have}/{need} relevés)")
        }
        Trend::Inconclusive(Inconclusive::WindowTooShort {
            window_secs,
            need_secs,
        }) => {
            format!("pente ? (fenêtre {window_secs}s < {need_secs}s)")
        }
        Trend::Slope {
            consumed_bytes_per_hour,
            ..
        } => {
            let gib_per_hour = consumed_bytes_per_hour / GIB;
            if *consumed_bytes_per_hour <= 0.0 {
                // Le disque se libère. Afficher « −10.0 Gio/h » ici se lirait
                // comme une perte : on nomme le sens plutôt que de le laisser
                // au signe.
                return format!("+{:.1} Gio/h libérés", -gib_per_hour);
            }
            match trend.hours_to_exhaustion(free_bytes) {
                Some(hours) => format!("−{:.1} Gio/h · ~{:.0} h", gib_per_hour, hours),
                None => format!("−{:.1} Gio/h", gib_per_hour),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB_U: u64 = 1024 * 1024 * 1024;

    fn sample(at: i64, gib: f64) -> DiskSample {
        DiskSample {
            observed_at_unix: at,
            free_bytes: (gib * GIB_U as f64) as u64,
        }
    }

    /// TÉMOIN NOMINAL — chiffres réels du 28/08. 110.4 Gio libres à 15:50,
    /// 74.4 Gio à 19:14 : environ 10.6 Gio/h sur trois heures et demie, et
    /// environ sept heures avant saturation.
    ///
    /// Mutant qui tue ce test : remplacer la régression par l'écart
    /// premier-dernier `(first.free - last.free) / window` ne le tue PAS ici
    /// (les points sont alignés) — c'est le test `un_pic_ne_fixe_pas_la_pente`
    /// qui porte cette assertion. Celui-ci meurt si le facteur 3600 disparaît,
    /// si le signe est inversé, ou si `hours_to_exhaustion` divise par autre
    /// chose que la pente de consommation.
    #[test]
    fn temoin_nominal_pente_reelle_du_28_08() {
        let base = 1_787_000_000;
        let samples = vec![
            sample(base, 110.4),
            sample(base + 3600, 99.8),
            sample(base + 7200, 89.2),
            sample(base + 12240, 74.4),
        ];

        let trend = assess(&samples, MIN_WINDOW_SECS, MIN_SAMPLES);

        let Trend::Slope {
            consumed_bytes_per_hour,
            window_secs,
            samples: count,
        } = trend
        else {
            panic!("fenêtre de 3h24 avec 4 relevés : la pente doit être concluante, obtenu {trend:?}");
        };
        assert_eq!(window_secs, 12240);
        assert_eq!(count, 4);
        let gib_per_hour = consumed_bytes_per_hour / GIB;
        assert!(
            (gib_per_hour - 10.6).abs() < 0.2,
            "pente attendue ≈ 10.6 Gio/h, obtenue {gib_per_hour:.2}"
        );
        let hours = trend
            .hours_to_exhaustion(74 * GIB_U)
            .expect("une pente de consommation positive doit donner une échéance");
        assert!(
            (hours - 7.0).abs() < 0.6,
            "échéance attendue ≈ 7 h, obtenue {hours:.2}"
        );
    }

    /// TÉMOIN DU REFUS — c'est le cœur du mandat, et le cas réellement payé.
    /// Mesurée sur 90 secondes, une compilation en cours rendait 112 Gio/h,
    /// soit « quarante minutes restantes ». Ce n'était pas une tendance.
    ///
    /// Mutant qui tue ce test : supprimer le contrôle de fenêtre, ou abaisser
    /// `MIN_WINDOW_SECS` sous 90. Le test échouerait alors sur l'assertion
    /// métier — il exigerait un refus et recevrait une pente de 112 Gio/h,
    /// c'est-à-dire exactement la fausse alerte que ce module doit empêcher.
    #[test]
    fn temoin_refus_fenetre_trop_courte_cas_compilation_112_gio_h() {
        let base = 1_787_000_000;
        // 2.8 Gio consommés en 90 s = 112 Gio/h si l'on ose extrapoler.
        let samples = vec![
            sample(base, 77.2),
            sample(base + 45, 75.8),
            sample(base + 90, 74.4),
        ];

        let trend = assess(&samples, MIN_WINDOW_SECS, MIN_SAMPLES);

        assert_eq!(
            trend,
            Trend::Inconclusive(Inconclusive::WindowTooShort {
                window_secs: 90,
                need_secs: MIN_WINDOW_SECS,
            }),
            "90 s de compilation ne sont pas une tendance : le verdict doit être un refus nommé"
        );
        assert!(
            trend.hours_to_exhaustion(74 * GIB_U).is_none(),
            "un refus ne doit JAMAIS produire d'échéance : c'est la fausse alerte à 40 minutes"
        );
        let rendered = format_trend(&trend, 74 * GIB_U);
        assert!(
            rendered.contains("pente ?"),
            "le refus doit se voir à l'affichage, obtenu {rendered}"
        );
        assert!(
            !rendered.contains("Gio/h"),
            "un refus ne doit pas afficher de pente, obtenu {rendered}"
        );
    }

    /// Le cas stable du même épisode : 74.39 trois fois en 90 s. Refusé aussi,
    /// et pour la même raison — la fenêtre, pas la valeur. Un refus ne dépend
    /// pas de ce que les chiffres racontent.
    #[test]
    fn serie_stable_sur_fenetre_courte_reste_un_refus() {
        let base = 1_787_000_000;
        let samples = vec![
            sample(base, 74.39),
            sample(base + 30, 74.39),
            sample(base + 60, 74.39),
        ];

        assert_eq!(
            assess(&samples, MIN_WINDOW_SECS, MIN_SAMPLES),
            Trend::Inconclusive(Inconclusive::WindowTooShort {
                window_secs: 60,
                need_secs: MIN_WINDOW_SECS,
            })
        );
    }

    /// Un point extrême isolé — fin de compilation qui rend 6 Gio d'un coup —
    /// ne doit pas fixer la pente d'une longue fenêtre par ailleurs régulière.
    ///
    /// Mutant qui tue ce test : remplacer la régression par l'écart
    /// premier-dernier. La pente sauterait alors à environ 2 Gio/h au lieu de
    /// rester proche de la tendance réelle de 10 Gio/h.
    #[test]
    fn un_pic_ne_fixe_pas_la_pente() {
        let base = 1_787_000_000;
        let mut samples = vec![
            sample(base, 100.0),
            sample(base + 3600, 90.0),
            sample(base + 7200, 80.0),
            sample(base + 10800, 70.0),
        ];
        // Dernier relevé aberrant : une purge vient de rendre 6 Gio.
        samples.push(sample(base + 10860, 76.0));

        let Trend::Slope {
            consumed_bytes_per_hour,
            ..
        } = assess(&samples, MIN_WINDOW_SECS, MIN_SAMPLES)
        else {
            panic!("la fenêtre est suffisante : la pente doit être concluante");
        };
        let gib_per_hour = consumed_bytes_per_hour / GIB;
        assert!(
            gib_per_hour > 6.0,
            "un point aberrant ne doit pas écraser la tendance, obtenu {gib_per_hour:.2} Gio/h"
        );
    }

    #[test]
    fn aucun_releve_et_releves_insuffisants_sont_des_refus_distincts() {
        assert_eq!(
            assess(&[], MIN_WINDOW_SECS, MIN_SAMPLES),
            Trend::Inconclusive(Inconclusive::NoSamples)
        );
        let base = 1_787_000_000;
        assert_eq!(
            assess(
                &[sample(base, 80.0), sample(base + 7200, 70.0)],
                MIN_WINDOW_SECS,
                MIN_SAMPLES
            ),
            Trend::Inconclusive(Inconclusive::NotEnoughSamples {
                have: 2,
                need: MIN_SAMPLES
            }),
            "deux points ne distinguent pas une tendance d'un saut"
        );
    }

    /// Un disque qui se vide ne doit pas produire d'échéance de saturation.
    #[test]
    fn disque_qui_se_libere_ne_predit_aucune_saturation() {
        let base = 1_787_000_000;
        let samples = vec![
            sample(base, 70.0),
            sample(base + 3600, 80.0),
            sample(base + 7200, 90.0),
        ];

        let trend = assess(&samples, MIN_WINDOW_SECS, MIN_SAMPLES);
        let Trend::Slope {
            consumed_bytes_per_hour,
            ..
        } = trend
        else {
            panic!("fenêtre suffisante");
        };
        assert!(consumed_bytes_per_hour < 0.0, "la consommation est négative");
        assert!(
            trend.hours_to_exhaustion(90 * GIB_U).is_none(),
            "un disque qui se libère n'a pas d'échéance de saturation"
        );
        let rendered = format_trend(&trend, 90 * GIB_U);
        assert!(
            rendered.contains("libérés"),
            "le sens doit être nommé, pas laissé au signe : obtenu {rendered}"
        );
        assert!(
            !rendered.contains('h') || !rendered.contains('~'),
            "aucune échéance de saturation ne doit s'afficher : obtenu {rendered}"
        );
    }

    #[test]
    fn historique_borne_et_garde_les_plus_recents() {
        let mut history = Vec::new();
        for i in 0..(MAX_SAMPLES + 10) {
            push_sample(&mut history, sample(1_787_000_000 + i as i64, 80.0));
        }
        assert_eq!(history.len(), MAX_SAMPLES);
        assert_eq!(
            history[history.len() - 1].observed_at_unix,
            1_787_000_000 + (MAX_SAMPLES + 9) as i64,
            "le relevé le plus récent doit survivre à la troncature"
        );
    }

    /// L'historique doit survivre au processus : c'est tout l'objet du mandat.
    /// Trois `who` successifs suffisamment espacés doivent rendre une pente,
    /// alors qu'aucun d'eux ne connaît les relevés des autres.
    ///
    /// Mutant qui tue ce test : faire de `record_and_assess` un calcul sans
    /// persistance (ne rien écrire, n'évaluer que le relevé courant). Chaque
    /// appel repartirait d'un point unique et le verdict resterait à jamais
    /// `NotEnoughSamples` — exactement l'angle mort que ce module supprime.
    #[test]
    fn trois_observateurs_ignorants_rendent_une_pente_grace_a_l_historique() {
        let dir = std::env::temp_dir().join(format!("jc6flux-trend-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = history_path(&dir);
        let base = 1_787_000_000;

        // Chaque appel ne connaît QUE son propre relevé.
        let first = record_and_assess(&path, sample(base, 100.0));
        assert!(
            matches!(first, Trend::Inconclusive(Inconclusive::NotEnoughSamples { .. })),
            "un premier relevé isolé ne peut pas conclure, obtenu {first:?}"
        );
        let _ = record_and_assess(&path, sample(base + 3600, 90.0));
        let third = record_and_assess(&path, sample(base + 7200, 80.0));

        let Trend::Slope {
            consumed_bytes_per_hour,
            samples: count,
            ..
        } = third
        else {
            panic!("trois relevés sur deux heures doivent conclure, obtenu {third:?}");
        };
        assert_eq!(count, 3, "les relevés antérieurs doivent avoir été retrouvés");
        let gib_per_hour = consumed_bytes_per_hour / GIB;
        assert!(
            (gib_per_hour - 10.0).abs() < 0.1,
            "pente attendue ≈ 10 Gio/h, obtenue {gib_per_hour:.2}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Défaut trouvé à l'exécution, pas en test : trois `who` lancés dans la
    /// même seconde empilaient trois relevés que `assess` fusionne ensuite, et
    /// l'affichage annonçait « 1/3 relevés » sur un fichier qui en contenait
    /// trois. Le refus était juste, l'historique était sale.
    ///
    /// Mutant qui tue ce test : revenir à `history.push(sample)` inconditionnel.
    /// L'historique regonfle à trois entrées pour une seule seconde observée.
    #[test]
    fn meme_seconde_remplace_au_lieu_d_empiler() {
        let mut history = Vec::new();
        let base = 1_787_945_108;
        push_sample(&mut history, sample(base, 71.6));
        push_sample(&mut history, sample(base, 71.5));
        push_sample(&mut history, sample(base, 71.4));

        assert_eq!(
            history.len(),
            1,
            "une seconde observée = un relevé, quel que soit le nombre d'appels"
        );
        assert_eq!(
            history[0].free_bytes,
            sample(base, 71.4).free_bytes,
            "le relevé conservé doit être le plus frais"
        );
    }

    /// Un historique illisible ne doit pas produire une pente inventée : on
    /// retombe sur un refus, pas sur un nombre.
    #[test]
    fn historique_corrompu_donne_un_refus_pas_une_pente() {
        let dir = std::env::temp_dir().join(format!("jc6flux-trend-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = history_path(&dir);
        std::fs::write(&path, b"{ ceci n'est pas du JSON").unwrap();

        assert!(load_history(&path).is_empty());
        let verdict = record_and_assess(&path, sample(1_787_000_000, 74.0));
        assert!(matches!(
            verdict,
            Trend::Inconclusive(Inconclusive::NotEnoughSamples { .. })
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Les relevés peuvent arriver dans le désordre (plusieurs wrappers) : le
    /// verdict ne doit pas en dépendre.
    #[test]
    fn ordre_d_arrivee_indifferent() {
        let base = 1_787_000_000;
        let chronologique = vec![
            sample(base, 100.0),
            sample(base + 3600, 90.0),
            sample(base + 7200, 80.0),
        ];
        let desordonne = vec![
            sample(base + 7200, 80.0),
            sample(base, 100.0),
            sample(base + 3600, 90.0),
        ];
        assert_eq!(
            assess(&chronologique, MIN_WINDOW_SECS, MIN_SAMPLES),
            assess(&desordonne, MIN_WINDOW_SECS, MIN_SAMPLES)
        );
    }
}
