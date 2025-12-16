/*!
# Error Handling Module

Comprehensive error management for the PDW system using Rust's Result type
and structured error hierarchy.
*/

use thiserror::Error;

/// Main error type for PDW operations
#[derive(Error, Debug)]
pub enum PdwError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Excel processing error: {0}")]
    Excel(#[from] ExcelError),
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("ETL pipeline error: {0}")]
    Etl(#[from] EtlError),
    
    #[error("Report generation error: {0}")]
    Report(#[from] ReportError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Logging initialization error: {0}")]
    Logging(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid configuration format: {message}")]
    InvalidFormat { message: String },
    
    #[error("Missing required configuration: {field}")]
    MissingField { field: String },
    
    #[error("Invalid directory path: {path} - {reason}")]
    InvalidPath { path: String, reason: String },
    
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("INI parsing error: {0}")]
    IniParse(#[from] ini::Error),
}

/// Excel processing errors
#[derive(Error, Debug)]
pub enum ExcelError {
    #[error("Failed to open Excel file: {path} - {reason}")]
    FileOpen { path: String, reason: String },
    
    #[error("Sheet not found: {sheet_name}")]
    SheetNotFound { sheet_name: String },
    
    #[error("Invalid sheet structure in {sheet_name}: {reason}")]
    InvalidStructure { sheet_name: String, reason: String },
    
    #[error("Data type conversion error in {sheet_name} at row {row}, column {col}: {reason}")]
    DataConversion { 
        sheet_name: String, 
        row: usize, 
        col: usize, 
        reason: String 
    },
    
    #[error("Missing required column: {column} in sheet {sheet_name}")]
    MissingColumn { column: String, sheet_name: String },
    
    #[error("Calamine error: {0}")]
    Calamine(#[from] calamine::Error),
}

/// Database operation errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection failed: {path} - {reason}")]
    ConnectionFailed { path: String, reason: String },
    
    #[error("SQL execution error: {query} - {reason}")]
    SqlExecution { query: String, reason: String },
    
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("Schema validation error: {reason}")]
    SchemaValidation { reason: String },
    
    #[error("Data insertion error: {table} - {reason}")]
    DataInsertion { table: String, reason: String },
    
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// ETL pipeline errors
#[derive(Error, Debug)]
pub enum EtlError {
    #[error("Pipeline initialization failed: {reason}")]
    InitializationFailed { reason: String },
    
    #[error("Data extraction failed: {source} - {reason}")]
    ExtractionFailed { source: String, reason: String },
    
    #[error("Data transformation failed: {stage} - {reason}")]
    TransformationFailed { stage: String, reason: String },
    
    #[error("Data loading failed: {target} - {reason}")]
    LoadingFailed { target: String, reason: String },
    
    #[error("Validation failed: {check} - {reason}")]
    ValidationFailed { check: String, reason: String },
    
    #[error("Pipeline configuration error: {reason}")]
    ConfigurationError { reason: String },
}

/// Report generation errors
#[derive(Error, Debug)]
pub enum ReportError {
    #[error("Query processing error: {query_name} - {reason}")]
    QueryProcessing { query_name: String, reason: String },
    
    #[error("Report template error: {template} - {reason}")]
    TemplateError { template: String, reason: String },
    
    #[error("Output generation error: {format} - {reason}")]
    OutputGeneration { format: String, reason: String },
    
    #[error("YAML query file error: {path} - {reason}")]
    YamlQueryFile { path: String, reason: String },
    
    #[error("Export format not supported: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),
    
    #[error("Excel writer error: {0}")]
    ExcelWriter(#[from] rust_xlsxwriter::XlsxError),
    
    #[error("CSV writer error: {0}")]
    CsvWriter(#[from] csv::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),
}

/// Result type alias for PDW operations
pub type PdwResult<T> = Result<T, PdwError>;

impl PdwError {
    /// Create a configuration error for missing field
    pub fn missing_config_field(field: &str) -> Self {
        PdwError::Config(ConfigError::MissingField {
            field: field.to_string(),
        })
    }
    
    /// Create an Excel error for missing sheet
    pub fn sheet_not_found(sheet_name: &str) -> Self {
        PdwError::Excel(ExcelError::SheetNotFound {
            sheet_name: sheet_name.to_string(),
        })
    }
    
    /// Create a database error for SQL execution
    pub fn sql_execution_error(query: &str, reason: &str) -> Self {
        PdwError::Database(DatabaseError::SqlExecution {
            query: query.to_string(),
            reason: reason.to_string(),
        })
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            PdwError::Config(_) => false,  // Configuration errors are not recoverable
            PdwError::Excel(ExcelError::FileOpen { .. }) => false,  // File access errors
            PdwError::Database(DatabaseError::ConnectionFailed { .. }) => false,  // Connection errors
            PdwError::Io(_) => false,  // IO errors are generally not recoverable
            _ => true,  // Other errors might be recoverable
        }
    }
    
    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            PdwError::Config(ConfigError::FileNotFound { path }) => {
                format!("Configuration file not found: {}. Please check the file path.", path)
            }
            PdwError::Config(ConfigError::VersionMismatch { expected, found }) => {
                format!("Configuration version mismatch. Expected {}, but found {}. Please update your configuration file.", expected, found)
            }
            PdwError::Excel(ExcelError::FileOpen { path, .. }) => {
                format!("Cannot open Excel file: {}. Please ensure the file exists and is not open in another application.", path)
            }
            PdwError::Database(DatabaseError::ConnectionFailed { path, .. }) => {
                format!("Cannot connect to database: {}. Please check file permissions and disk space.", path)
            }
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let error = PdwError::missing_config_field("test_field");
        assert!(matches!(error, PdwError::Config(ConfigError::MissingField { .. })));
    }
    
    #[test]
    fn test_error_recoverability() {
        let config_error = PdwError::missing_config_field("test");
        assert!(!config_error.is_recoverable());
        
        let etl_error = PdwError::Etl(EtlError::ValidationFailed {
            check: "test".to_string(),
            reason: "test".to_string(),
        });
        assert!(etl_error.is_recoverable());
    }
    
    #[test]
    fn test_user_messages() {
        let error = PdwError::Config(ConfigError::FileNotFound {
            path: "test.toml".to_string(),
        });
        let message = error.user_message();
        assert!(message.contains("Configuration file not found"));
        assert!(message.contains("test.toml"));
    }
}