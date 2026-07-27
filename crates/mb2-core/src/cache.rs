use crate::error::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModMetadata {
    pub module_id: String,
    pub nexus_mod_id: Option<i64>,
    pub name: String,
    pub summary: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub updated_at: i64,
}

pub struct MetadataCache {
    conn: Connection,
}

impl MetadataCache {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS mod_metadata (
                module_id TEXT PRIMARY KEY,
                nexus_mod_id INTEGER,
                name TEXT NOT NULL,
                summary TEXT,
                version TEXT,
                url TEXT,
                updated_at INTEGER NOT NULL
            );",
        )?;

        Ok(Self { conn })
    }

    pub fn upsert(&self, entry: &CachedModMetadata) -> Result<()> {
        self.conn.execute(
            "INSERT INTO mod_metadata (module_id, nexus_mod_id, name, summary, version, url, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(module_id) DO UPDATE SET
                nexus_mod_id = excluded.nexus_mod_id,
                name = excluded.name,
                summary = excluded.summary,
                version = excluded.version,
                url = excluded.url,
                updated_at = excluded.updated_at",
            params![
                entry.module_id,
                entry.nexus_mod_id,
                entry.name,
                entry.summary,
                entry.version,
                entry.url,
                entry.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get(&self, module_id: &str) -> Result<Option<CachedModMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT module_id, nexus_mod_id, name, summary, version, url, updated_at
             FROM mod_metadata WHERE module_id = ?1",
        )?;

        let mut rows = stmt.query([module_id])?;
        if let Some(row) = rows.next()? {
            return Ok(Some(CachedModMetadata {
                module_id: row.get(0)?,
                nexus_mod_id: row.get(1)?,
                name: row.get(2)?,
                summary: row.get(3)?,
                version: row.get(4)?,
                url: row.get(5)?,
                updated_at: row.get(6)?,
            }));
        }

        Ok(None)
    }

    pub fn search(&self, query: &str) -> Result<Vec<CachedModMetadata>> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT module_id, nexus_mod_id, name, summary, version, url, updated_at
             FROM mod_metadata
             WHERE name LIKE ?1 OR module_id LIKE ?1 OR summary LIKE ?1
             ORDER BY name LIMIT 50",
        )?;

        let rows = stmt.query_map([&pattern], |row| {
            Ok(CachedModMetadata {
                module_id: row.get(0)?,
                nexus_mod_id: row.get(1)?,
                name: row.get(2)?,
                summary: row.get(3)?,
                version: row.get(4)?,
                url: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
