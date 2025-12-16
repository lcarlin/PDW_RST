/*!
# Database Management Module

Handles SQLite operations including connection management, schema creation,
and data operations. Maintains compatibility with Python PDW database structure.
*/

use crate::error::{DatabaseError, PdwError};
use crate::excel::Transaction;
use rusqlite::{Connection, params, Result as SqliteResult, Row};
use std::path::Path;
use chrono::NaiveDate;
use serde_json::Value;

/// Database manager for SQLite operations
pub struct DatabaseManager {
    connection: Connection,
}

/// Processed transaction with enriched temporal data
#[derive(Debug, Clone)]
pub struct ProcessedTransaction {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub transaction_type: String,
    pub description: String,
    pub credit: f64,
    pub debit: f64,
    pub month: String,
    pub year: String,
    pub month_name: String,
    pub year_month: String,
    pub origin: String,
}

impl DatabaseManager {
    /// Create new database connection
    pub fn new(db_path: &Path) -> Result<Self, PdwError> {
        let connection = Connection::open(db_path)
            .map_err(|e| DatabaseError::ConnectionFailed {
                path: db_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(Self { connection })
    }
    
    /// Create all required database tables
    pub fn create_tables(&self) -> Result<(), PdwError> {
        // Main entries table (identical to Python version)
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS LANCAMENTOS_GERAIS (
                Data DATE,
                DIA_SEMANA TEXT,
                TIPO TEXT,
                DESCRICAO TEXT,
                Credito REAL,
                Debito REAL,
                Mes TEXT,
                Ano TEXT,
                MES_EXTENSO TEXT,
                AnoMes TEXT,
                Origem TEXT
            )",
            [],
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "CREATE TABLE LANCAMENTOS_GERAIS".to_string(),
            reason: e.to_string(),
        })?;
        
        // Transaction types table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS TiposLancamentos (
                Código TEXT,
                Descrição TEXT
            )",
            [],
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "CREATE TABLE TiposLancamentos".to_string(),
            reason: e.to_string(),
        })?;
        
        // Guiding table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS GUIDING (
                TABLE_NAME TEXT,
                ACCOUNTING TEXT,
                LOADABLE TEXT
            )",
            [],
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "CREATE TABLE GUIDING".to_string(),
            reason: e.to_string(),
        })?;
        
        // Installments table
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS PARCELAMENTOS (
                Data DATE,
                'Tipo Lançamento' TEXT,
                Descricao TEXT,
                Debito REAL
            )",
            [],
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "CREATE TABLE PARCELAMENTOS".to_string(),
            reason: e.to_string(),
        })?;
        
        Ok(())
    }
    
    /// Drop table if exists
    pub fn drop_table(&self, table_name: &str) -> Result<(), PdwError> {
        let query = format!("DROP TABLE IF EXISTS {}", table_name);
        self.connection.execute(&query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: query.clone(),
                reason: e.to_string(),
            })?;
        Ok(())
    }
    
    /// Insert processed transactions
    pub fn insert_transactions(&self, transactions: &[ProcessedTransaction]) -> Result<usize, PdwError> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO LANCAMENTOS_GERAIS 
             (Data, DIA_SEMANA, TIPO, DESCRICAO, Credito, Debito, Mes, Ano, MES_EXTENSO, AnoMes, Origem)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "INSERT INTO LANCAMENTOS_GERAIS".to_string(),
            reason: e.to_string(),
        })?;
        
        let mut count = 0;
        for transaction in transactions {
            stmt.execute(params![
                transaction.date.format("%Y-%m-%d").to_string(),
                transaction.day_of_week,
                transaction.transaction_type,
                transaction.description,
                transaction.credit,
                transaction.debit,
                transaction.month,
                transaction.year,
                transaction.month_name,
                transaction.year_month,
                transaction.origin,
            ]).map_err(|e| DatabaseError::DataInsertion {
                table: "LANCAMENTOS_GERAIS".to_string(),
                reason: e.to_string(),
            })?;
            count += 1;
        }
        
        Ok(count)
    }
    
    /// Insert reference data
    pub fn insert_reference_data(&self, table_name: &str, data: &[Vec<String>]) -> Result<usize, PdwError> {
        if data.is_empty() {
            return Ok(0);
        }
        
        // Create table dynamically based on data structure
        let column_count = data[0].len();
        let columns: Vec<String> = (1..=column_count)
            .map(|i| format!("col{} TEXT", i))
            .collect();
        
        let create_query = format!(
            "CREATE TABLE IF NOT EXISTS {} ({})",
            table_name,
            columns.join(", ")
        );
        
        self.connection.execute(&create_query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: create_query,
                reason: e.to_string(),
            })?;
        
        // Insert data
        let placeholders: Vec<String> = (1..=column_count)
            .map(|i| format!("?{}", i))
            .collect();
        
        let insert_query = format!(
            "INSERT INTO {} VALUES ({})",
            table_name,
            placeholders.join(", ")
        );
        
        let mut stmt = self.connection.prepare(&insert_query)
            .map_err(|e| DatabaseError::SqlExecution {
                query: insert_query.clone(),
                reason: e.to_string(),
            })?;
        
        let mut count = 0;
        for row in data {
            let params: Vec<&dyn rusqlite::ToSql> = row.iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect();
            
            stmt.execute(&params[..])
                .map_err(|e| DatabaseError::DataInsertion {
                    table: table_name.to_string(),
                    reason: e.to_string(),
                })?;
            count += 1;
        }
        
        Ok(count)
    }
    
    /// Execute SQL query and return results
    pub fn execute_query(&self, sql: &str) -> Result<Vec<Vec<Value>>, PdwError> {
        let mut stmt = self.connection.prepare(sql)
            .map_err(|e| DatabaseError::SqlExecution {
                query: sql.to_string(),
                reason: e.to_string(),
            })?;
        
        let column_count = stmt.column_count();
        let rows = stmt.query_map([], |row| {
            let mut values = Vec::new();
            for i in 0..column_count {
                let value: rusqlite::types::Value = row.get(i)?;
                let json_value = match value {
                    rusqlite::types::Value::Null => Value::Null,
                    rusqlite::types::Value::Integer(i) => Value::Number(i.into()),
                    rusqlite::types::Value::Real(f) => {
                        Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| 0.into()))
                    }
                    rusqlite::types::Value::Text(s) => Value::String(s),
                    rusqlite::types::Value::Blob(_) => Value::String("BLOB".to_string()),
                };
                values.push(json_value);
            }
            Ok(values)
        }).map_err(|e| DatabaseError::SqlExecution {
            query: sql.to_string(),
            reason: e.to_string(),
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| DatabaseError::SqlExecution {
                query: sql.to_string(),
                reason: e.to_string(),
            })?);
        }
        
        Ok(results)
    }
    
    /// Create pivot tables for historical analysis
    pub fn create_pivot_tables(&self, entries_table: &str, types_table: &str, 
                              full_pivot_table: &str, annual_pivot_table: &str) -> Result<(), PdwError> {
        
        // Get transaction types for column ordering
        let types_query = format!("SELECT Descrição FROM {}", types_table);
        let types_result = self.execute_query(&types_query)?;
        
        // Create monthly pivot table
        self.create_monthly_pivot(entries_table, full_pivot_table, &types_result)?;
        
        // Create annual pivot table  
        self.create_annual_pivot(entries_table, annual_pivot_table, &types_result)?;
        
        Ok(())
    }
    
    /// Create monthly pivot table
    fn create_monthly_pivot(&self, entries_table: &str, pivot_table: &str, 
                           types: &[Vec<Value>]) -> Result<(), PdwError> {
        
        // Drop existing table
        self.drop_table(pivot_table)?;
        
        // Build dynamic pivot query
        let mut columns = vec!["AnoMes TEXT".to_string()];
        let mut select_columns = vec!["AnoMes".to_string()];
        
        for type_row in types {
            if let Some(Value::String(type_name)) = type_row.get(0) {
                let safe_name = type_name.replace(" ", "_").replace("'", "");
                columns.push(format!("[{}] REAL", type_name));
                select_columns.push(format!(
                    "COALESCE(SUM(CASE WHEN TIPO = '{}' THEN Debito ELSE 0 END), 0) AS [{}]",
                    type_name, type_name
                ));
            }
        }
        
        // Create table
        let create_query = format!(
            "CREATE TABLE {} ({})",
            pivot_table,
            columns.join(", ")
        );
        
        self.connection.execute(&create_query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: create_query,
                reason: e.to_string(),
            })?;
        
        // Insert pivot data
        let insert_query = format!(
            "INSERT INTO {} SELECT {} FROM {} GROUP BY AnoMes ORDER BY AnoMes",
            pivot_table,
            select_columns.join(", "),
            entries_table
        );
        
        self.connection.execute(&insert_query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: insert_query,
                reason: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Create annual pivot table
    fn create_annual_pivot(&self, entries_table: &str, pivot_table: &str, 
                          types: &[Vec<Value>]) -> Result<(), PdwError> {
        
        // Drop existing table
        self.drop_table(pivot_table)?;
        
        // Build dynamic pivot query
        let mut columns = vec!["Ano TEXT".to_string()];
        let mut select_columns = vec!["Ano".to_string()];
        
        for type_row in types {
            if let Some(Value::String(type_name)) = type_row.get(0) {
                columns.push(format!("[{}] REAL", type_name));
                select_columns.push(format!(
                    "COALESCE(SUM(CASE WHEN TIPO = '{}' THEN Debito ELSE 0 END), 0) AS [{}]",
                    type_name, type_name
                ));
            }
        }
        
        // Create table
        let create_query = format!(
            "CREATE TABLE {} ({})",
            pivot_table,
            columns.join(", ")
        );
        
        self.connection.execute(&create_query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: create_query,
                reason: e.to_string(),
            })?;
        
        // Insert pivot data
        let insert_query = format!(
            "INSERT INTO {} SELECT {} FROM {} GROUP BY Ano ORDER BY Ano",
            pivot_table,
            select_columns.join(", "),
            entries_table
        );
        
        self.connection.execute(&insert_query, [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: insert_query,
                reason: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Perform data validation and cleanup
    pub fn validate_and_clean_data(&self, entries_table: &str, types_table: &str, 
                                  save_discarded: bool, discarded_table: &str) -> Result<(), PdwError> {
        
        if save_discarded {
            // Save discarded data
            let save_query = format!(
                "CREATE TABLE IF NOT EXISTS {} AS SELECT * FROM {} WHERE (Data IS NULL OR TIPO IS NULL)",
                discarded_table, entries_table
            );
            self.connection.execute(&save_query, [])
                .map_err(|e| DatabaseError::SqlExecution {
                    query: save_query,
                    reason: e.to_string(),
                })?;
        }
        
        // Remove invalid records
        let cleanup_queries = vec![
            format!("DELETE FROM {} WHERE (Data IS NULL OR TIPO IS NULL)", entries_table),
            format!("DELETE FROM {} WHERE (Código IS NULL OR Descrição IS NULL)", types_table),
            "DELETE FROM PARCELAMENTOS WHERE (DATA IS NULL OR \"Tipo Lançamento\" IS NULL)".to_string(),
        ];
        
        for query in cleanup_queries {
            self.connection.execute(&query, [])
                .map_err(|e| DatabaseError::SqlExecution {
                    query: query.clone(),
                    reason: e.to_string(),
                })?;
        }
        
        // Create origins view
        self.connection.execute("DROP VIEW IF EXISTS Origens", [])
            .map_err(|e| DatabaseError::SqlExecution {
                query: "DROP VIEW Origens".to_string(),
                reason: e.to_string(),
            })?;
        
        self.connection.execute(
            "CREATE VIEW Origens AS 
             SELECT TABLE_NAME as nome FROM GUIDING 
             WHERE LOADABLE = 'X' AND ACCOUNTING = 'X'",
            []
        ).map_err(|e| DatabaseError::SqlExecution {
            query: "CREATE VIEW Origens".to_string(),
            reason: e.to_string(),
        })?;
        
        Ok(())
    }
    
    /// Get connection reference for advanced operations
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

/// Trait for database operations
pub trait DatabaseOperations {
    fn create_connection(db_path: &Path) -> Result<Self, PdwError>
    where
        Self: Sized;
    
    fn create_tables(&self) -> Result<(), PdwError>;
    fn insert_transactions(&self, transactions: &[ProcessedTransaction]) -> Result<usize, PdwError>;
    fn execute_query(&self, sql: &str) -> Result<Vec<Vec<Value>>, PdwError>;
}

impl DatabaseOperations for DatabaseManager {
    fn create_connection(db_path: &Path) -> Result<Self, PdwError> {
        Self::new(db_path)
    }
    
    fn create_tables(&self) -> Result<(), PdwError> {
        self.create_tables()
    }
    
    fn insert_transactions(&self, transactions: &[ProcessedTransaction]) -> Result<usize, PdwError> {
        self.insert_transactions(transactions)
    }
    
    fn execute_query(&self, sql: &str) -> Result<Vec<Vec<Value>>, PdwError> {
        self.execute_query(sql)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::NaiveDate;
    
    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = DatabaseManager::new(&db_path).unwrap();
        assert!(db_path.exists());
    }
    
    #[test]
    fn test_table_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = DatabaseManager::new(&db_path).unwrap();
        db.create_tables().unwrap();
        
        // Verify tables exist
        let result = db.execute_query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='LANCAMENTOS_GERAIS'"
        ).unwrap();
        assert!(!result.is_empty());
    }
    
    #[test]
    fn test_transaction_insertion() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = DatabaseManager::new(&db_path).unwrap();
        db.create_tables().unwrap();
        
        let transactions = vec![
            ProcessedTransaction {
                date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                day_of_week: "Segunda-feira".to_string(),
                transaction_type: "ALM".to_string(),
                description: "Test transaction".to_string(),
                credit: 0.0,
                debit: 100.0,
                month: "01".to_string(),
                year: "2024".to_string(),
                month_name: "01-Janeiro".to_string(),
                year_month: "2024/01".to_string(),
                origin: "TestSheet".to_string(),
            }
        ];
        
        let count = db.insert_transactions(&transactions).unwrap();
        assert_eq!(count, 1);
    }
    
    #[test]
    fn test_query_execution() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let db = DatabaseManager::new(&db_path).unwrap();
        db.create_tables().unwrap();
        
        let result = db.execute_query("SELECT COUNT(*) FROM LANCAMENTOS_GERAIS").unwrap();
        assert_eq!(result.len(), 1);
    }
}