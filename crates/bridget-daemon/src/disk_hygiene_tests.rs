    use super::*;
    use std::collections::HashMap;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    struct MapProbe {
        map: HashMap<PathBuf, Option<Vec<u32>>>,
    }

    impl OwnerProbe for MapProbe {
        fn owner_pids(&self, path: &Path) -> Option<Vec<u32>> {
            self.map.get(path).cloned().unwrap_or(Some(vec![]))
        }
    }

    struct CountingSlowProbe {
        calls: AtomicUsize,
    }

    impl OwnerProbe for CountingSlowProbe {
        fn owner_pids(&self, _path: &Path) -> Option<Vec<u32>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            // 50 ms × 1000 = 50 s si le filtre d'âge est sauté — le test exige <5 s.
            std::thread::sleep(Duration::from_millis(50));
            Some(vec![])
        }
    }

    fn set_mtime(path: &Path, unix_secs: i64) {
        use std::io::Write;
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        let _ = (&file).write_all(b"x");
        let times = libc::timeval {
            tv_sec: unix_secs,
            tv_usec: 0,
        };
        let c_path = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        // SAFETY: chemin CString valide ; tableau de 2 timeval (atime, mtime).
        unsafe {
            let tv = [times, times];
            assert_eq!(libc::utimes(c_path.as_ptr(), tv.as_ptr()), 0);
        }
    }

    #[test]
    fn orpheline_agee_est_ramassee() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS + 1, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Delete
        );
    }

    #[test]
    fn copie_du_jour_est_epargnee() {
        assert_eq!(
            decide_purge(60, Some(&[]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "copie_du_jour"
            }
        );
    }

    #[test]
    fn processus_vivant_est_epargne() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, Some(&[4242]), ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "processus_vivant"
            }
        );
    }

    #[test]
    fn proprietaire_inconnu_fail_closed() {
        assert_eq!(
            decide_purge(ORPHAN_MIN_AGE_SECS * 2, None, ORPHAN_MIN_AGE_SECS),
            PurgeDecision::Spare {
                reason: "proprietaire_inconnu"
            }
        );
    }

    #[test]
    fn seuil_franchi_produit_warn() {
        let message = disk_pressure_warning(5 * 1024 * 1024 * 1024, DISK_WARN_FREE_BYTES).unwrap();
        assert!(message.contains("WARN disque"));
        assert!(message.contains("< 20 Gi"));
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES, DISK_WARN_FREE_BYTES).is_none());
        assert!(disk_pressure_warning(DISK_WARN_FREE_BYTES + 1, DISK_WARN_FREE_BYTES).is_none());
    }

    #[test]
    fn purge_reelle_respecte_age_et_vivant() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();

        let aged = root.join("bridget-aged-orphan");
        let today = root.join("bridget-today");
        let live = root.join("bridget-live-owner");
        let now = 1_700_000_000i64;
        set_mtime(&aged, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);
        set_mtime(&today, now - 120);
        set_mtime(&live, now - (ORPHAN_MIN_AGE_SECS as i64) - 60);

        let mut map = HashMap::new();
        map.insert(aged.clone(), Some(vec![]));
        map.insert(today.clone(), Some(vec![]));
        map.insert(live.clone(), Some(vec![7]));
        let probe = MapProbe { map };

        let report =
            purge_orphan_bridget_tmp_at(&root, now as u64, ORPHAN_MIN_AGE_SECS, usize::MAX, &probe);

        assert!(!aged.exists(), "orpheline âgée doit être ramassée");
        assert!(today.exists(), "copie du jour doit être épargnée");
        assert!(live.exists(), "copie d'un vivant doit être épargnée");
        assert_eq!(report.deleted, vec![aged]);
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &today && *r == "copie_du_jour")
        );
        assert!(
            report
                .spared
                .iter()
                .any(|(p, r)| p == &live && *r == "processus_vivant")
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn boot_avec_1000_jeunes_reste_sous_5s_sans_sonde() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-boot-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let now = 1_700_000_000u64;
        for i in 0..1000 {
            let path = root.join(format!("bridget-jeune-{i}"));
            set_mtime(&path, (now as i64) - 30);
        }
        let probe = CountingSlowProbe {
            calls: AtomicUsize::new(0),
        };
        let started = Instant::now();
        let report = purge_orphan_bridget_tmp_at(
            &root,
            now,
            ORPHAN_MIN_AGE_SECS,
            MAX_ENTRIES_PER_PASS,
            &probe,
        );
        let elapsed = started.elapsed();
        assert_eq!(
            probe.calls.load(Ordering::SeqCst),
            0,
            "aucune sonde sur des copies du jour"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "boot simulé trop lent: {elapsed:?}"
        );
        assert!(report.deleted.is_empty());
        assert_eq!(report.spared.len(), 1000);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn borne_reporte_le_surplus_age() {
        let root =
            std::env::temp_dir().join(format!("bridget-hygiene-borne-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let now = 1_700_000_000i64;
        for i in 0..5 {
            set_mtime(
                &root.join(format!("bridget-age-{i}")),
                now - (ORPHAN_MIN_AGE_SECS as i64) - 60,
            );
        }
        let probe = MapProbe {
            map: HashMap::new(),
        };
        let report = purge_orphan_bridget_tmp_at(&root, now as u64, ORPHAN_MIN_AGE_SECS, 2, &probe);
        assert_eq!(report.deleted.len(), 2);
        assert_eq!(report.deferred, 3);
        let _ = fs::remove_dir_all(&root);
    }
