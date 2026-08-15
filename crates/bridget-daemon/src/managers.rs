//! Managers spécialisés pour séparer les responsabilités du daemon

use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Gère les connexions actives des wrappers
pub struct ConnectionManager {
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    conn_counter: u64,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
            conn_counter: 0,
        }
    }

    pub fn register_connection(&mut self, conn_id: String, writer: BufWriter<UnixStream>) -> u64 {
        let conn_id = conn_id.clone();
        self.connections.insert(conn_id.clone(), Arc::new(Mutex::new(writer)));
        self.conn_counter += 1;
        self.conn_counter
    }

    pub fn set_connection_info(&mut self, conn_id: &str, name: String, host: String, os: String, instance_id: Option<String>) {
        self.conn_names.insert(conn_id.to_string(), name);
        self.conn_hosts.insert(conn_id.to_string(), host);
        self.conn_operating_systems.insert(conn_id.to_string(), os);
        if let Some(instance_id) = instance_id {
            self.conn_instances.insert(conn_id.to_string(), instance_id);
        }
    }

    pub fn remove_connection(&mut self, conn_id: &str) -> Option<Arc<Mutex<BufWriter<UnixStream>>>> {
        let writer_opt = self.connections.remove(conn_id);
        self.conn_names.remove(conn_id);
        self.conn_hosts.remove(conn_id);
        self.conn_operating_systems.remove(conn_id);
        self.conn_instances.remove(conn_id);
        writer_opt
    }

    pub fn get_writer(&self, conn_id: &str) -> Option<Arc<Mutex<BufWriter<UnixStream>>>> {
        self.connections.get(conn_id).cloned()
    }

    pub fn get_name(&self, conn_id: &str) -> Option<&String> {
        self.conn_names.get(conn_id)
    }

    pub fn get_host(&self, conn_id: &str) -> Option<&String> {
        self.conn_hosts.get(conn_id)
    }

    pub fn get_os(&self, conn_id: &str) -> Option<&String> {
        self.conn_operating_systems.get(conn_id)
    }

    pub fn iter_connections(&self) -> impl Iterator<Item = (&String, &String)> {
        self.conn_names.iter()
    }
}

/// Gère la présence durable des agents (reconnexion, état)
pub struct PresenceManager {
    presences: HashMap<String, Presence>,
}

#[derive(Clone, Debug)]
pub struct Presence {
    pub name: String,
    pub agent_type: String,
    pub host: String,
    pub os: String,
    pub transport: String,
    pub state: String,
    pub last_seen: Instant,
    pub reconnect_count: u32,
}

impl PresenceManager {
    pub fn new() -> Self {
        Self {
            presences: HashMap::new(),
        }
    }

    pub fn register_presence(&mut self, instance_id: String, presence: Presence) {
        self.presences.insert(instance_id, presence);
    }

    pub fn update_presence(&mut self, instance_id: &str, state: String) {
        if let Some(presence) = self.presences.get_mut(instance_id) {
            presence.state = state;
            presence.last_seen = Instant::now();
        }
    }

    pub fn remove_presence(&mut self, instance_id: &str) {
        self.presences.remove(instance_id);
    }

    pub fn mark_unreachable(&mut self, conn_id: &str, conn_instances: &HashMap<String, String>) {
        if let Some(instance_id) = conn_instances.get(conn_id) {
            if let Some(presence) = self.presences.get_mut(instance_id) {
                presence.state = "unreachable".to_string();
            }
        }
    }

    pub fn get_presence(&self, instance_id: &str) -> Option<&Presence> {
        self.presences.get(instance_id)
    }

    pub fn get_reconnect_count(&self, instance_id: &str) -> u32 {
        self.presences
            .get(instance_id)
            .map(|p| p.reconnect_count)
            .unwrap_or(0)
    }

    pub fn iter_presences(&self) -> impl Iterator<Item = &Presence> {
        self.presences.values()
    }
}

/// Gère les demandes suivies et leur escalade
pub struct RequestManager {
    pending_replies: Vec<PendingReply>,
}

#[derive(Clone, Debug)]
pub struct PendingReply {
    pub msg_id: String,
    pub sender: String,
    pub target: String,
    pub created_at: Instant,
    pub timeout_secs: u64,
    pub escalation_level: u32,
}

impl RequestManager {
    pub fn new() -> Self {
        Self {
            pending_replies: Vec::new(),
        }
    }

    pub fn add_request(&mut self, request: PendingReply) {
        self.pending_replies.push(request);
    }

    pub fn remove_request(&mut self, msg_id: &str) -> bool {
        if let Some(pos) = self.pending_replies.iter().position(|r| r.msg_id == msg_id) {
            self.pending_replies.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_requests_by_sender(&self, sender: &str) -> Vec<&PendingReply> {
        self.pending_replies
            .iter()
            .filter(|r| r.sender == sender)
            .collect()
    }

    pub fn get_request(&self, msg_id: &str) -> Option<&PendingReply> {
        self.pending_replies.iter().find(|r| r.msg_id == msg_id)
    }

    pub fn iter_mut_requests(&mut self) -> impl Iterator<Item = &mut PendingReply> {
        self.pending_replies.iter_mut()
    }

    pub fn cleanup_expired(&mut self, max_age_secs: u64) {
        let now = Instant::now();
        self.pending_replies.retain(|r| {
            let elapsed = now.duration_since(r.created_at).as_secs();
            elapsed <= max_age_secs
        });
    }
}
