//! Réponses ordinaires conservées avant envoi, jusqu'au dépôt durable du maître.
use bridget_core::BridgetMessage;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::path::Path;

pub(crate) struct ReplyOutbox {
    conn: rusqlite::Connection,
    instance: String,
}
impl ReplyOutbox {
    pub(crate) fn open(state: &Path, instance: &str) -> Result<Self, String> {
        let key = format!("{:x}", Sha256::digest(instance.as_bytes()));
        let conn = crate::delivery_spool244::private_connection(
            &state
                .join("native-replies244")
                .join(format!("{key}.sqlite")),
        )?;
        conn.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS replies244(id TEXT PRIMARY KEY, instance TEXT NOT NULL, created_at INTEGER NOT NULL, message BLOB NOT NULL, blocked INTEGER NOT NULL DEFAULT 0);").map_err(|e|e.to_string())?;
        let foreign: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM replies244 WHERE instance<>?1)",
                [instance],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if foreign {
            return Err("reply_outbox_instance_changed".into());
        }
        Ok(Self {
            conn,
            instance: instance.into(),
        })
    }
    pub(crate) fn retain(
        &self,
        original: &BridgetMessage,
        mut reply: BridgetMessage,
        now: i64,
    ) -> Result<BridgetMessage, String> {
        let identity = serde_json::to_vec(&(&self.instance, &original.id, &original.from))
            .map_err(|e| e.to_string())?;
        reply.id = format!("reply244-{}", format!("{:x}", Sha256::digest(identity)));
        let bytes = serde_json::to_vec(&reply).map_err(|e| e.to_string())?;
        if bytes.len() >= bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES {
            return Err("reply_outbox_frame_too_large".into());
        }
        let previous: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT message FROM replies244 WHERE id=?1",
                [&reply.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if let Some(previous) = previous {
            // Les champs temporels du nouveau constructeur ne changent pas la
            // réponse déjà conservée. Un autre corps reste un conflit explicite.
            let saved: BridgetMessage =
                serde_json::from_slice(&previous).map_err(|e| e.to_string())?;
            if saved.from != reply.from
                || saved.to != reply.to
                || saved.body != reply.body
                || saved.in_reply_to != reply.in_reply_to
            {
                return Err("reply_outbox_envelope_changed".into());
            }
            return Ok(saved);
        }
        self.conn
            .execute(
                "INSERT INTO replies244(id,instance,created_at,message) VALUES(?1,?2,?3,?4)",
                params![reply.id, self.instance, now, bytes],
            )
            .map_err(|e| e.to_string())?;
        Ok(reply)
    }
    pub(crate) fn pending(&self, now: i64) -> Result<Option<BridgetMessage>, String> {
        // Après l'horizon d'idempotence, conserver la preuve sans réémettre.
        let row:Option<(String,usize)>=self.conn.query_row("SELECT id,length(message) FROM replies244 WHERE blocked=0 AND created_at>?1 ORDER BY created_at,id LIMIT 1",[now.saturating_sub(7*86400)],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(|e|e.to_string())?;
        let Some((id, len)) = row else {
            return Ok(None);
        };
        if len >= bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES {
            return Err("reply_outbox_frame_corrupt".into());
        }
        let bytes: Vec<u8> = self
            .conn
            .query_row("SELECT message FROM replies244 WHERE id=?1", [&id], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        let msg: BridgetMessage = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if msg.id != id || msg.reply || msg.in_reply_to.is_none() {
            return Err("reply_outbox_frame_corrupt".into());
        }
        Ok(Some(msg))
    }
    pub(crate) fn refused(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE replies244 SET blocked=1 WHERE id=?1 AND instance=?2",
                params![id, self.instance],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub(crate) fn acknowledged(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM replies244 WHERE id=?1 AND instance=?2",
                params![id, self.instance],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spec244_reponse_conservee_reouverte_et_accusee_sans_rejouer_la_demande() {
        let root = std::env::temp_dir().join(format!("reply244-{}", uuid::Uuid::new_v4()));
        let original = BridgetMessage::new("parent", "child", "question");
        let mut reply = BridgetMessage::new("child", "parent", "réponse");
        reply.in_reply_to = Some(original.id.clone());
        let out = ReplyOutbox::open(&root, "instance").unwrap();
        let saved = out.retain(&original, reply.clone(), 1000).unwrap();
        let mut changed = reply.clone();
        changed.body = "autre réponse".into();
        assert!(out.retain(&original, changed, 1001).is_err());
        drop(out);
        let out = ReplyOutbox::open(&root, "instance").unwrap();
        assert_eq!(out.retain(&original, reply, 1002).unwrap().id, saved.id);
        assert_eq!(out.pending(1003).unwrap().unwrap().id, saved.id);
        assert!(out.pending(1000 + 7 * 86400).unwrap().is_none());
        assert!(out.pending(1003).unwrap().is_some());
        out.acknowledged("autre-id").unwrap();
        assert!(out.pending(1003).unwrap().is_some());
        out.acknowledged(&saved.id).unwrap();
        out.acknowledged(&saved.id).unwrap();
        assert!(out.pending(1003).unwrap().is_none());
        drop(out);
        std::fs::remove_dir_all(root).unwrap();
    }
}
