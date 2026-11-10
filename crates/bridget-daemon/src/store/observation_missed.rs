//! Faits manqués158B sur Store existant : aucune E/S fournisseur.
//! Dépôt O(F+B*(log F+Cowner)), F/O<=8192, B<=128 ;
//! Cowner=O(O*(log F+log O)) au pire pour une nouvelle attribution ;
//! page O(log F+P), P<=64 et64Kio ; résumé O(log F+faits owner), sans corps.
use super::{Store, StoreError};
use crate::observation::{NoticePayload, Notification};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

// Conversion SQL canonique : plus de40 sites locaux, aucun nouveau type erreur.
impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

pub(crate) const MAX_MISSED_FACT_BYTES: usize = 16 * 1024;
const RETENTION: i64 = 7 * 24 * 3600;
const MAX_FACTS: i64 = 8192;
const MAX_OWNERS: i64 = 8192;
const MAX_SUB: i64 = 64;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ObservationWriteOutcome {
    Written,
    FailedBeforeWrite,
    WriteUnknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ObservationSummary {
    pub owner: String,
    pub summary_id: String,
    pub key: String,
    pub episode: u64,
    pub generation: u64,
    pub issued_at: i64,
    pub expires_at: i64,
    pub fact_seq: u64,
    pub owner_gap_version: u64,
    pub global_gap_version: u64,
    pub body: String,
    pub phase: String,
    pub delivery_id: Option<String>,
    pub recipient_instance_id: Option<String>,
    pub delivery_generation: Option<u64>,
}

pub(crate) fn ensure_schema(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS observation_missed_owners (
        kind TEXT NOT NULL CHECK(kind IN ('owner','global')), owner TEXT NOT NULL,
        last_fact_or_gap_at INTEGER NOT NULL, gap_count INTEGER NOT NULL DEFAULT 0,
        dropped_count INTEGER NOT NULL DEFAULT 0, coverage_count INTEGER NOT NULL DEFAULT 0,
        gap_from INTEGER, gap_to INTEGER, gap_version INTEGER NOT NULL DEFAULT 0,
        available INTEGER, return_episode INTEGER NOT NULL DEFAULT 0,
        frozen_episode INTEGER NOT NULL DEFAULT 0, return_generation INTEGER NOT NULL DEFAULT 0,
        confirmed_fact_seq INTEGER NOT NULL DEFAULT 0,
        confirmed_owner_gap_version INTEGER NOT NULL DEFAULT 0,
        confirmed_global_gap_version INTEGER NOT NULL DEFAULT 0,
        summary_id TEXT UNIQUE, summary_json BLOB CHECK(summary_json IS NULL OR length(summary_json)<=16384),
        summary_phase TEXT CHECK(summary_phase IN ('prepared','dispatching','confirmed','outcome_unknown')),
        summary_expires_at INTEGER, summary_delivery_id TEXT,
        summary_instance TEXT, summary_delivery_generation INTEGER,
        CHECK((kind='global' AND owner='') OR (kind='owner' AND length(CAST(owner AS BLOB)) BETWEEN 1 AND 128)),
        PRIMARY KEY(kind,owner));
        CREATE TABLE IF NOT EXISTS observation_missed (
        fact_seq INTEGER PRIMARY KEY AUTOINCREMENT,
        fact_id TEXT NOT NULL UNIQUE, owner_kind TEXT NOT NULL DEFAULT 'owner' CHECK(owner_kind='owner'),
        owner TEXT NOT NULL, subscription_id TEXT NOT NULL,
        observed_at INTEGER NOT NULL, deadline_at INTEGER NOT NULL, retained_until INTEGER NOT NULL,
        event_name TEXT, fact_json BLOB NOT NULL CHECK(length(CAST(fact_json AS BLOB))<=16384),
        phase TEXT NOT NULL CHECK(phase IN ('pending_write','missed_before_write','write_unknown')),
        failure_code TEXT,
        FOREIGN KEY(owner_kind,owner) REFERENCES observation_missed_owners(kind,owner) ON DELETE CASCADE);
        CREATE INDEX IF NOT EXISTS observation_missed_owner_seq ON observation_missed(owner,fact_seq);
        CREATE INDEX IF NOT EXISTS observation_missed_subscription_seq ON observation_missed(subscription_id,fact_seq);
        CREATE INDEX IF NOT EXISTS observation_missed_retention ON observation_missed(retained_until,fact_seq);
        CREATE INDEX IF NOT EXISTS observation_missed_pending ON observation_missed(phase,deadline_at);")
        .map_err(StoreError::Sqlite)
}

// Trois mutations réelles : source/oversize, éviction/TTL, perte d'attribution.
fn gap(
    tx: &Transaction<'_>,
    owner: Option<&str>,
    count: i64,
    dropped: i64,
    coverage: i64,
    from: i64,
    to: i64,
) -> Result<(), StoreError> {
    let (kind, who) = owner.map_or(("global", ""), |v| ("owner", v));
    tx.execute(
        "INSERT INTO observation_missed_owners(kind,owner,last_fact_or_gap_at)
        VALUES(?1,?2,?3) ON CONFLICT(kind,owner) DO NOTHING",
        params![kind, who, to],
    )?;
    tx.execute(
        "UPDATE observation_missed_owners SET gap_count=gap_count+?3,
        dropped_count=dropped_count+?4, coverage_count=coverage_count+?5,
        gap_version=gap_version+1, gap_from=MIN(COALESCE(gap_from,?6),?6),
        gap_to=MAX(COALESCE(gap_to,?7),?7),last_fact_or_gap_at=MAX(last_fact_or_gap_at,?7)
        WHERE kind=?1 AND owner=?2",
        params![kind, who, count, dropped, coverage, from, to],
    )?;
    Ok(())
}

fn ensure_owner(tx: &Transaction<'_>, owner: &str, now: i64) -> Result<bool, StoreError> {
    if owner.is_empty() || owner.len() > 128 {
        return Err(StoreError::Invariant("owner missed invalide"));
    }
    if tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM observation_missed_owners WHERE kind='owner' AND owner=?1)",
        [owner],
        |r| r.get::<_, bool>(0),
    )? {
        return Ok(true);
    }
    let count: i64 = tx.query_row(
        "SELECT COUNT(*) FROM observation_missed_owners WHERE kind='owner'",
        [],
        |r| r.get(0),
    )?;
    if count >= MAX_OWNERS {
        let old: Option<(String, i64)> = tx
            .query_row(
                "SELECT owner,last_fact_or_gap_at FROM observation_missed_owners o
            WHERE kind='owner' AND COALESCE(summary_phase,'') NOT IN ('prepared','dispatching')
            AND NOT EXISTS(SELECT 1 FROM observation_missed f WHERE f.owner=o.owner)
            ORDER BY last_fact_or_gap_at,owner LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((who, at)) = old else {
            gap(tx, None, 1, 0, 0, now, now)?;
            return Ok(false);
        };
        gap(tx, None, 0, 0, 1, at, now)?;
        tx.execute(
            "DELETE FROM observation_missed_owners WHERE kind='owner' AND owner=?1",
            [who],
        )?;
    }
    tx.execute("INSERT INTO observation_missed_owners(kind,owner,last_fact_or_gap_at) VALUES('owner',?1,?2)", params![owner,now])?;
    Ok(true)
}

fn load_summary(conn: &Connection, id: &str) -> Result<Option<ObservationSummary>, StoreError> {
    let row=conn.query_row("SELECT summary_json,summary_phase,summary_delivery_id,summary_instance,summary_delivery_generation
        FROM observation_missed_owners WHERE kind='owner' AND summary_id=?1",[id],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<u64>>(4)?))).optional()?;
    row.map(|(bytes, phase, id, instance, generation)| {
        let mut summary: ObservationSummary = serde_json::from_slice(&bytes)
            .map_err(|_| StoreError::Invariant("resume missed illisible"))?;
        summary.phase = phase;
        summary.delivery_id = id;
        summary.recipient_instance_id = instance;
        summary.delivery_generation = generation;
        Ok(summary)
    })
    .transpose()
}

impl Store {
    /// O(F+B*(log F+Cowner)), Cowner=O(O*(log F+log O)), F/O<=8192.
    /// Snapshotonce et provenance committés avant enqueue, même sans avis.
    pub(crate) fn save_observation_batch(
        &mut self,
        snapshot: Option<&[u8]>,
        notices: &[Notification],
        now: i64,
    ) -> Result<Vec<String>, StoreError> {
        if notices.is_empty() && snapshot.is_none() {
            return Ok(Vec::new());
        }
        if snapshot.is_some_and(|b| b.len() > 262144) {
            return Err(StoreError::Invariant("snapshot observation trop grand"));
        }
        let timeout: u64 = self
            .conn
            .query_row("PRAGMA busy_timeout", [], |r| r.get(0))?;
        self.conn.busy_timeout(std::time::Duration::ZERO)?;
        let result = (|| {
            let tx = self
                .conn
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            if let Some(bytes) = snapshot {
                tx.execute("INSERT INTO observation_subscriptions(singleton,payload)VALUES(1,?1) ON CONFLICT(singleton)DO UPDATE SET payload=excluded.payload",[bytes])?;
            }
            let mut total: i64 =
                tx.query_row("SELECT COUNT(*) FROM observation_missed", [], |r| r.get(0))?;
            let mut counts: HashMap<String, i64> = {
                let mut q=tx.prepare("SELECT subscription_id,COUNT(*) FROM observation_missed GROUP BY subscription_id")?;
                q.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<Result<_, _>>()?
            };
            let mut accepted = Vec::new();
            for notice in notices {
                if notice.subscription_id.is_empty()
                    || notice.subscription_id.len() > 128
                    || uuid::Uuid::parse_str(&notice.fact_id).is_err()
                {
                    return Err(StoreError::Invariant("provenance missed invalide"));
                }
                let exists: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM observation_missed WHERE fact_id=?1)",
                    [&notice.fact_id],
                    |r| r.get(0),
                )?;
                if exists {
                    continue;
                }
                if !ensure_owner(&tx, &notice.owner, now)? {
                    continue;
                }
                let retained = json!({"fact_id":notice.fact_id,"owner":notice.owner,"subscription_id":notice.subscription_id,"payload":notice.payload});
                let bytes = serde_json::to_vec(&retained)
                    .map_err(|_| StoreError::Invariant("fait missed non serialisable"))?;
                if bytes.len() > MAX_MISSED_FACT_BYTES {
                    gap(&tx, Some(&notice.owner), 1, 0, 0, now, now)?;
                    continue;
                }
                let event = match &notice.payload {
                    NoticePayload::Fact(fact) => Some(
                        serde_json::to_value(fact.event)
                            .map_err(|_| StoreError::Invariant("type observation invalide"))?
                            .as_str()
                            .ok_or(StoreError::Invariant("type observation invalide"))?
                            .to_owned(),
                    ),
                    NoticePayload::Gap {
                        kind: _,
                        source: _,
                        since_secs,
                        dropped,
                    } => {
                        let since = now.saturating_sub((*since_secs).min(i64::MAX as u64) as i64);
                        gap(
                            &tx,
                            Some(&notice.owner),
                            1,
                            (*dropped).min(i64::MAX as u64) as i64,
                            0,
                            since,
                            now,
                        )?;
                        None
                    }
                };
                while counts.get(&notice.subscription_id).copied().unwrap_or(0) >= MAX_SUB
                    || total >= MAX_FACTS
                {
                    let by_sub =
                        counts.get(&notice.subscription_id).copied().unwrap_or(0) >= MAX_SUB;
                    let old = if by_sub {
                        tx.query_row("SELECT fact_seq,owner,subscription_id,observed_at FROM observation_missed WHERE subscription_id=?1 ORDER BY fact_seq LIMIT 1",[&notice.subscription_id],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?)))?
                    } else {
                        tx.query_row("SELECT fact_seq,owner,subscription_id,observed_at FROM observation_missed ORDER BY fact_seq LIMIT 1",[],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?)))?
                    };
                    gap(&tx, Some(&old.1), 1, 0, 0, old.3, old.3)?;
                    tx.execute("DELETE FROM observation_missed WHERE fact_seq=?1", [old.0])?;
                    *counts.entry(old.2).or_default() -= 1;
                    total -= 1;
                }
                tx.execute("INSERT INTO observation_missed(fact_id,owner,subscription_id,observed_at,deadline_at,retained_until,event_name,fact_json,phase)
                VALUES(?1,?2,?3,?4,?5,?6,?7,?8,'pending_write')",params![notice.fact_id,notice.owner,notice.subscription_id,now,now.saturating_add(5),now.saturating_add(RETENTION),event,bytes])?;
                tx.execute("UPDATE observation_missed_owners SET last_fact_or_gap_at=MAX(last_fact_or_gap_at,?2) WHERE kind='owner' AND owner=?1",params![notice.owner,now])?;
                total += 1;
                *counts.entry(notice.subscription_id.clone()).or_default() += 1;
                accepted.push(notice.fact_id.clone());
            }
            tx.commit()?;
            Ok(accepted)
        })();
        let restored = self
            .conn
            .busy_timeout(std::time::Duration::from_millis(timeout));
        match (result, restored) {
            (Err(error), _) => Err(error),
            (Ok(_), Err(error)) => Err(StoreError::Sqlite(error)),
            (Ok(ids), Ok(())) => Ok(ids),
        }
    }

    /// O(log F), callback corrélé ; aucune recréation d'un fait évincé.
    pub(crate) fn finish_observation_write(
        &mut self,
        owner: &str,
        id: &str,
        outcome: ObservationWriteOutcome,
        _now: i64,
    ) -> Result<bool, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=match outcome {
            ObservationWriteOutcome::Written=>tx.execute("DELETE FROM observation_missed WHERE owner=?1 AND fact_id=?2 AND phase='pending_write'",params![owner,id])?,
            ObservationWriteOutcome::FailedBeforeWrite=>tx.execute("UPDATE observation_missed SET phase='missed_before_write',failure_code='before_write' WHERE owner=?1 AND fact_id=?2 AND phase='pending_write'",params![owner,id])?,
            ObservationWriteOutcome::WriteUnknown=>tx.execute("UPDATE observation_missed SET phase='write_unknown',failure_code='write_unknown' WHERE owner=?1 AND fact_id=?2 AND phase='pending_write'",params![owner,id])?,
        };
        tx.commit()?;
        Ok(changed > 0)
    }

    /// O(F+O log F), F/O<=8192 ; restart est exclusivement le bootstrap ROOT.
    pub(crate) fn maintain_observation_missed(
        &mut self,
        now: i64,
        restart: bool,
    ) -> Result<usize, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut changed = if restart {
            tx.execute(
                "UPDATE observation_missed_owners SET available=0 WHERE kind='owner'",
                [],
            )?;
            tx.execute("UPDATE observation_missed SET phase='write_unknown',failure_code='daemon_restart' WHERE phase='pending_write'",[])?
        } else {
            tx.execute("UPDATE observation_missed SET phase='write_unknown',failure_code='deadline_unknown' WHERE phase='pending_write' AND deadline_at<=?1",[now])?
        };
        let lost = {
            let mut q=tx.prepare("SELECT owner,COUNT(*),MIN(observed_at),MAX(observed_at) FROM observation_missed WHERE retained_until<=?1 GROUP BY owner")?;
            q.query_map([now], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?
        };
        for (owner, count, from, to) in lost {
            gap(&tx, Some(&owner), count, 0, 0, from, to)?;
        }
        changed += tx.execute(
            "DELETE FROM observation_missed WHERE retained_until<=?1",
            [now],
        )?;
        let expired = {
            let mut q=tx.prepare("SELECT owner,last_fact_or_gap_at FROM observation_missed_owners o WHERE kind='owner'
            AND last_fact_or_gap_at<=?1 AND COALESCE(summary_phase,'') NOT IN('prepared','dispatching')
            AND NOT EXISTS(SELECT 1 FROM observation_missed f WHERE f.owner=o.owner)")?;
            q.query_map([now.saturating_sub(RETENTION)], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?
        };
        for (owner, at) in expired {
            gap(&tx, None, 0, 0, 1, at, now)?;
            changed += tx.execute(
                "DELETE FROM observation_missed_owners WHERE kind='owner' AND owner=?1",
                [owner],
            )?;
        }
        tx.commit()?;
        Ok(changed)
    }

    /// O(log F+P), P<=64, page entière<=65536octets. Aucune mutation/TTLread.
    pub(crate) fn observation_missed_page(
        &self,
        owner: &str,
        limit: Option<u32>,
        after_seq: Option<u64>,
        now: i64,
    ) -> Result<Value, StoreError> {
        let limit = limit.unwrap_or(20);
        if !(1..=64).contains(&limit) || after_seq.is_some_and(|n| n > i64::MAX as u64) {
            return Err(StoreError::Invariant("page missed invalide"));
        }
        let gaps = {
            let mut q=self.conn.prepare("SELECT kind,gap_count,dropped_count,coverage_count,gap_from,gap_to,gap_version FROM observation_missed_owners WHERE (kind='owner' AND owner=?1) OR kind='global' ORDER BY kind")?;
            q.query_map([owner],|r|Ok(json!({"kind":r.get::<_,String>(0)?,"count":r.get::<_,i64>(1)?,"dropped":r.get::<_,i64>(2)?,"coverage_evicted":r.get::<_,i64>(3)?,"from":r.get::<_,Option<i64>>(4)?,"through":r.get::<_,Option<i64>>(5)?,"version":r.get::<_,i64>(6)?})))?.collect::<Result<Vec<_>,_>>()?
        };
        let mut page =
            json!({"status":"ok","facts":[],"gaps":gaps,"next_after_seq":null,"has_more":false});
        let mut q=self.conn.prepare("SELECT fact_seq,fact_id,subscription_id,observed_at,phase,failure_code,fact_json FROM observation_missed WHERE owner=?1 AND fact_seq>?2 AND retained_until>?3 ORDER BY fact_seq LIMIT ?4")?;
        let rows = q.query_map(
            params![owner, after_seq.unwrap_or(0), now, limit + 1],
            |r| {
                Ok((
                    r.get::<_, u64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, Vec<u8>>(6)?,
                ))
            },
        )?;
        let mut used = serde_json::to_vec(&page)
            .map_err(|_| StoreError::Invariant("page missed non serialisable"))?
            .len()
            + 128;
        let mut facts = Vec::new();
        let mut last = None;
        for row in rows {
            let (seq, id, sub, at, phase, failure, bytes) = row?;
            let payload: Value = serde_json::from_slice(&bytes)
                .map_err(|_| StoreError::Invariant("fait missed illisible"))?;
            let item = json!({"fact_seq":seq,"fact_id":id,"subscription_id":sub,"observed_at":at,"phase":phase,"failure_code":failure,"fact":payload});
            let item_bytes = serde_json::to_vec(&item)
                .map_err(|_| StoreError::Invariant("page missed non serialisable"))?
                .len()
                + 1;
            if facts.len() >= limit as usize || used.saturating_add(item_bytes) > 65536 {
                page["has_more"] = json!(true);
                break;
            }
            used += item_bytes;
            facts.push(item);
            last = Some(seq);
        }
        page["facts"] = json!(facts);
        page["next_after_seq"] = json!(last);
        Ok(page)
    }

    /// O(log O), seul false→true ouvre un épisode ; première vue initialise.
    pub(crate) fn observation_owner_presence(
        &mut self,
        owner: &str,
        available: bool,
        _now: i64,
    ) -> Result<bool, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let previous: Option<Option<bool>> = tx
            .query_row(
                "SELECT available FROM observation_missed_owners WHERE kind='owner' AND owner=?1",
                [owner],
                |r| r.get(0),
            )
            .optional()?;
        let Some(previous) = previous else {
            return Ok(false);
        };
        let returned = previous == Some(false) && available;
        tx.execute("UPDATE observation_missed_owners SET available=?2,return_episode=return_episode+?3 WHERE kind='owner' AND owner=?1",params![owner,available,i64::from(returned)])?;
        tx.commit()?;
        Ok(returned)
    }

    /// O(log F+Fowner), agrégat fixe ; tous les pendingowner doivent être soldés.
    pub(crate) fn freeze_observation_summary(
        &mut self,
        owner: &str,
        now: i64,
    ) -> Result<Option<ObservationSummary>, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let row=tx.query_row("SELECT return_episode,frozen_episode,return_generation,confirmed_fact_seq,confirmed_owner_gap_version,confirmed_global_gap_version,gap_version,gap_count,dropped_count,coverage_count,gap_from,gap_to,summary_id,summary_phase FROM observation_missed_owners WHERE kind='owner' AND owner=?1",[owner],|r|Ok((r.get::<_,u64>(0)?,r.get::<_,u64>(1)?,r.get::<_,u64>(2)?,r.get::<_,u64>(3)?,r.get::<_,u64>(4)?,r.get::<_,u64>(5)?,r.get::<_,u64>(6)?,r.get::<_,i64>(7)?,r.get::<_,i64>(8)?,r.get::<_,i64>(9)?,r.get::<_,Option<i64>>(10)?,r.get::<_,Option<i64>>(11)?,r.get::<_,Option<String>>(12)?,r.get::<_,Option<String>>(13)?))).optional()?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row
            .13
            .as_deref()
            .is_some_and(|p| p == "prepared" || p == "dispatching")
        {
            let id = row.12.ok_or(StoreError::Invariant("resume sans UUID"))?;
            tx.commit()?;
            return load_summary(&self.conn, &id);
        }
        if row.0 <= row.1 {
            return Ok(None);
        }
        let pending:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM observation_missed WHERE owner=?1 AND phase='pending_write')",[owner],|r|r.get(0))?;
        if pending {
            return Ok(None);
        }
        let global=tx.query_row("SELECT gap_version,gap_count,dropped_count,coverage_count,gap_from,gap_to FROM observation_missed_owners WHERE kind='global'",[],|r|Ok((r.get::<_,u64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,Option<i64>>(4)?,r.get::<_,Option<i64>>(5)?))).optional()?.unwrap_or((0,0,0,0,None,None));
        let (through,from,to,count):(u64,Option<i64>,Option<i64>,i64)=tx.query_row("SELECT COALESCE(MAX(fact_seq),?2),MIN(observed_at),MAX(observed_at),COUNT(*) FROM observation_missed WHERE owner=?1 AND fact_seq>?2",params![owner,row.3],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        if count == 0 && row.6 <= row.4 && global.0 <= row.5 {
            tx.execute("UPDATE observation_missed_owners SET frozen_episode=?2 WHERE kind='owner' AND owner=?1",params![owner,row.0])?;
            tx.commit()?;
            return Ok(None);
        }
        let counts = {
            let mut q=tx.prepare("SELECT event_name,COUNT(*) FROM observation_missed WHERE owner=?1 AND fact_seq>?2 AND event_name IS NOT NULL GROUP BY event_name")?;
            q.query_map(params![owner, row.3], |r| {
                Ok((r.get::<_, String>(0)?, json!(r.get::<_, i64>(1)?)))
            })?
            .collect::<Result<serde_json::Map<String, Value>, _>>()?
        };
        let body=json!({"notice":"faits manques, pas validation de mission ; details via events missed","count":count,"types":counts,"from":from,"through":to,
            "owner_gap":if row.6>row.4 {json!({"version":row.6,"count":row.7,"dropped":row.8,"coverage_evicted":row.9,"from":row.10,"through":row.11})}else{Value::Null},
            "global_gap":if global.0>row.5 {json!({"version":global.0,"count":global.1,"dropped":global.2,"coverage_evicted":global.3,"from":global.4,"through":global.5})}else{Value::Null},
            "previous_unknown":row.13.as_deref()==Some("outcome_unknown")}).to_string();
        let id = uuid::Uuid::new_v4().to_string();
        let generation = row
            .2
            .checked_add(1)
            .ok_or(StoreError::Invariant("generation resume epuisee"))?;
        let summary = ObservationSummary {
            owner: owner.into(),
            summary_id: id.clone(),
            key: format!("missed-summary:{id}"),
            episode: row.0,
            generation,
            issued_at: now,
            expires_at: now.saturating_add(600),
            fact_seq: through,
            owner_gap_version: row.6,
            global_gap_version: global.0,
            body,
            phase: "prepared".into(),
            delivery_id: None,
            recipient_instance_id: None,
            delivery_generation: None,
        };
        let bytes = serde_json::to_vec(&summary)
            .map_err(|_| StoreError::Invariant("resume non serialisable"))?;
        if bytes.len() > 16384 {
            return Err(StoreError::Invariant("resume trop grand"));
        }
        tx.execute("UPDATE observation_missed_owners SET frozen_episode=?2,return_generation=?3,summary_id=?4,summary_json=?5,summary_phase='prepared',summary_expires_at=?6,summary_delivery_id=NULL,summary_instance=NULL,summary_delivery_generation=NULL WHERE kind='owner' AND owner=?1",params![owner,row.0,generation,id,bytes,summary.expires_at])?;
        tx.commit()?;
        Ok(Some(summary))
    }

    /// O(O), O<=8192, sans copie des corps de faits.
    pub(crate) fn observation_summaries_pending(
        &self,
    ) -> Result<Vec<ObservationSummary>, StoreError> {
        let mut q=self.conn.prepare("SELECT summary_json,summary_phase,summary_delivery_id,summary_instance,summary_delivery_generation FROM observation_missed_owners WHERE kind='owner' AND summary_phase IN('prepared','dispatching') ORDER BY owner")?;
        let rows = q.query_map([], |r| {
            Ok((
                r.get::<_, Vec<u8>>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<u64>>(4)?,
            ))
        })?;
        let mut output = Vec::new();
        for row in rows {
            let (bytes, phase, id, instance, generation) = row?;
            let mut summary: ObservationSummary = serde_json::from_slice(&bytes)
                .map_err(|_| StoreError::Invariant("resume missed illisible"))?;
            summary.phase = phase;
            summary.delivery_id = id;
            summary.recipient_instance_id = instance;
            summary.delivery_generation = generation;
            output.push(summary);
        }
        Ok(output)
    }

    /// O(log O+B), B<=16Kio ; lecture courante, même après outcome_unknown.
    pub(crate) fn observation_summary_current(
        &self,
        owner: &str,
    ) -> Result<Option<ObservationSummary>, StoreError> {
        let id = self
            .conn
            .query_row(
                "SELECT summary_id FROM observation_missed_owners WHERE kind='owner' AND owner=?1",
                [owner],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        match id {
            Some(id) => load_summary(&self.conn, &id),
            None => Ok(None),
        }
    }

    /// O(log O), preuves de remise immuables ; aucun appel099 dans cette TX.
    pub(crate) fn bind_observation_summary_delivery(
        &mut self,
        id: &str,
        delivery_id: &str,
        instance: &str,
        generation: u64,
    ) -> Result<bool, StoreError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=tx.execute("UPDATE observation_missed_owners SET summary_phase='dispatching',summary_delivery_id=?2,summary_instance=?3,summary_delivery_generation=?4 WHERE kind='owner' AND summary_id=?1 AND summary_phase='prepared' AND summary_delivery_id IS NULL",params![id,delivery_id,instance,generation])?;
        let same:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM observation_missed_owners WHERE kind='owner' AND summary_id=?1 AND summary_delivery_id=?2 AND summary_instance=?3 AND summary_delivery_generation=?4)",params![id,delivery_id,instance,generation],|r|r.get(0))?;
        tx.commit()?;
        Ok(changed > 0 || same)
    }

    /// O(log O), seulement après ACK099Ok ; MAX sur les trois cutoffs GELÉS.
    pub(crate) fn confirm_observation_summary(
        &mut self,
        owner: &str,
        id: &str,
        generation: u64,
        delivery_id: &str,
        instance: &str,
        delivery_generation: u64,
    ) -> Result<bool, StoreError> {
        let Some(summary) = load_summary(&self.conn, id)? else {
            return Ok(false);
        };
        if summary.owner != owner
            || summary.generation != generation
            || summary.delivery_id.as_deref() != Some(delivery_id)
            || summary.recipient_instance_id.as_deref() != Some(instance)
            || summary.delivery_generation != Some(delivery_generation)
        {
            return Ok(false);
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let changed=tx.execute("UPDATE observation_missed_owners SET summary_phase='confirmed',confirmed_fact_seq=MAX(confirmed_fact_seq,?7),confirmed_owner_gap_version=MAX(confirmed_owner_gap_version,?8),confirmed_global_gap_version=MAX(confirmed_global_gap_version,?9) WHERE kind='owner' AND owner=?1 AND summary_id=?2 AND return_generation=?3 AND summary_delivery_id=?4 AND summary_instance=?5 AND summary_delivery_generation=?6 AND summary_phase IN('dispatching','outcome_unknown')",params![owner,id,generation,delivery_id,instance,delivery_generation,summary.fact_seq,summary.owner_gap_version,summary.global_gap_version])?;
        tx.commit()?;
        Ok(changed > 0)
    }

    /// O(log O), ROOT recherche d'abord le reçu099acked, jamais de cutoff ici.
    pub(crate) fn expire_observation_summary(
        &mut self,
        id: &str,
        now: i64,
    ) -> Result<bool, StoreError> {
        Ok(self.conn.execute("UPDATE observation_missed_owners SET summary_phase='outcome_unknown' WHERE kind='owner' AND summary_id=?1 AND summary_phase IN('prepared','dispatching') AND summary_expires_at<=?2",params![id,now])?>0)
    }
}

#[cfg(test)]
#[path = "observation_missed_tests.rs"]
mod tests;
