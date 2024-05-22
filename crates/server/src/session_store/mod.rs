use axum::async_trait;
use rusqlite::{params, OptionalExtension};
use thiserror::Error;
use tower_sessions::{cookie::time::OffsetDateTime, session::{Id, Record}, session_store, ExpiredDeletion, SessionStore};
use deadpool_sqlite::{Object, Pool};

const DEFAULT_TABLE_NAME: &'static str = "__tower_sessions";

#[derive(Debug, Error)]
pub enum DeadpoolSqliteStoreError {
    #[error("Deadpool interact error: {0}")]
    DeadpoolInteract(#[from] deadpool_sqlite::InteractError),
    #[error("Deadpool pool error: {0}")]
    DeadpoolPool(#[from] deadpool_sqlite::PoolError),
    #[error("Rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Serde json decode error: {0}")]
    JsonDecode(serde_json::Error),
    #[error("Serde json encode error: {0}")]
    JsonEncode(serde_json::Error),
}

impl From<DeadpoolSqliteStoreError> for session_store::Error {
    fn from (err: DeadpoolSqliteStoreError) -> Self {
        use DeadpoolSqliteStoreError::*;
        use session_store::Error;

        match err {
            JsonEncode(inner) => Error::Encode(inner.to_string()),
            JsonDecode(inner) => Error::Decode(inner.to_string()),
            other => Error::Backend(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeadpoolSqliteStore {
    pool: Pool,
    table_name: String,
}
impl DeadpoolSqliteStore {
    pub fn new(pool: Pool) -> Self {
        Self::new_with_table_name(pool, DEFAULT_TABLE_NAME).unwrap()
    }

    pub fn new_with_table_name<T: Into<String>>(pool: Pool, table_name: T) -> Result<Self, String> {
        let table_name = table_name.into();

        if table_name.is_empty() || !table_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            Err("Table name is not valid. Can only contain ascii alphanumeric, - and _".to_string())?;
        }
        
        Ok(Self {
            pool,
            table_name,
        })
    }

    pub async fn get_conn(&self) -> Result<Object, session_store::Error> {
        Ok(self.pool.get().await
            .map_err(DeadpoolSqliteStoreError::from)?)
    }

    pub async fn migrate(&self) -> Result<(), session_store::Error> {
        let conn = self.get_conn().await?;

        let sql = format!(r#"
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY NOT NULL,
                data BLOB NOT NULL,
                expiry_date INTEGER NOT NULL
            );"#,
            self.table_name);

        conn.interact(move |conn| {
            conn.execute(&sql, ())
        }).await
        .map_err(DeadpoolSqliteStoreError::from)?
        .map_err(DeadpoolSqliteStoreError::from)?;
        
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for DeadpoolSqliteStore {
    async fn delete_expired(&self) -> Result<(), session_store::Error> {
        let sql = format!(r#"
            DELETE FROM {}
            WHERE expiry_date < ?1 
        "#, self.table_name);
        let now = OffsetDateTime::now_utc().unix_timestamp();
        
        let conn = self.get_conn().await?;
        
        conn.interact(move |conn| 
            conn.execute(&sql, params![now]))
            .await
            .map_err(DeadpoolSqliteStoreError::from)?
            .map_err(DeadpoolSqliteStoreError::from)?;
        
        Ok(())
    }
}

#[async_trait]
impl SessionStore for DeadpoolSqliteStore {
    async fn create(&self, record: &mut Record) -> Result<(), session_store::Error> {
        let exists_sql = format!(r#"SELECT 1 FROM {} WHERE id = ?1"#, self.table_name);
        let insert_sql = format!(r#"
            INSERT INTO {} (id, data, expiry_date)
            VALUES (?1, ?2, ?3);
        "#, self.table_name);

        let mut id = record.id.clone();
        let payload = serde_json::to_vec(&record)
            .map_err(|e| DeadpoolSqliteStoreError::JsonEncode(e))?;
        let expiry = record.expiry_date.unix_timestamp();
    
        let conn = self.get_conn().await?;
        let id = conn.interact(move |conn| {
            let tx = conn.transaction()?;

            {
                let mut exists_stmd = tx.prepare_cached(&exists_sql)?;

                // Re-key the record until we successfully find a unique ID
                while exists_stmd.exists(params![id.to_string()])? {
                    id = Id::default();
                }
            }

            {
                let mut insert_stmt = tx.prepare_cached(&insert_sql)?;

                insert_stmt.execute(params![
                    id.to_string(),
                    payload,
                    expiry,
                ])?;
            }

            tx.commit()?;

            Ok::<_, DeadpoolSqliteStoreError>(id)
        })
        .await
        .map_err(DeadpoolSqliteStoreError::from)?
        .map_err(DeadpoolSqliteStoreError::from)?;

        record.id = id;

        Ok(())
    }

    async fn save(&self, record: &Record) -> Result<(), session_store::Error> {
        let update_sql = format!(r#"
            UPDATE {} SET
                data = ?1, 
                expiry_date = ?2
            WHERE
                id = ?3;
        "#, self.table_name);

        let conn = self.get_conn().await?;
        

        let id = record.id.clone();
        let payload = serde_json::to_vec(&record)
                .map_err(|e| DeadpoolSqliteStoreError::JsonEncode(e))?;
        let expiry = record.expiry_date.unix_timestamp();

        conn.interact(move |conn| {
            let mut update_stmt = conn.prepare_cached(&update_sql)?;

            update_stmt.execute(params![
                payload,
                expiry,
                id.to_string(),
            ])?;

            Ok::<_, DeadpoolSqliteStoreError>(())
        })
        .await
        .map_err(DeadpoolSqliteStoreError::from)?
        .map_err(DeadpoolSqliteStoreError::from)?;

        Ok(())
    }

    async fn load(&self, id: &Id) -> Result<Option<Record>, session_store::Error> {
        let select_sql = format!(r#"
            SELECT data 
            FROM {} 
            WHERE 
                id = ?1
                AND expiry_date > ?2;
        "#, self.table_name);

        let conn = self.get_conn().await?;
        let id_string = id.to_string();
        let payload = conn.interact(move |conn| {
            let now = OffsetDateTime::now_utc().unix_timestamp();

            let mut select_stmt = conn.prepare_cached(&select_sql)?;

            let data = select_stmt.query_row(params![id_string, now], |row| 
                row.get::<_, Vec<u8>>(0))
                .optional()?;

            Ok::<_, DeadpoolSqliteStoreError>(data)
        })
        .await
        .map_err(DeadpoolSqliteStoreError::from)?
        .map_err(DeadpoolSqliteStoreError::from)?;

        let record = payload
            .map(|data| serde_json::from_slice::<Record>(&data))
            .transpose()
            .map_err(|e| DeadpoolSqliteStoreError::JsonDecode(e))?
            .map(|mut record| {
                // Make sure the id is updated after the re-keying done during insert
                record.id = id.to_owned();
                record
            });

        Ok(record)
    }

    async fn delete(&self, id: &Id) -> Result<(), session_store::Error> {
        let delete_sql = format!(r#"
            DELETE FROM {}
            WHERE id = ?1
        "#, self.table_name);

        let conn = self.get_conn().await?;
        let id_string = id.to_string();
        conn.interact(move |conn| {
            let mut delete_stmt = conn.prepare_cached(&delete_sql)?;

            delete_stmt.execute(params![id_string])?;

            Ok::<_, DeadpoolSqliteStoreError>(())
        })
        .await
        .map_err(DeadpoolSqliteStoreError::from)?
        .map_err(DeadpoolSqliteStoreError::from)?;

        Ok(())
    }
}

