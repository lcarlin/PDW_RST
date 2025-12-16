# Design Document - PDW Rust Migration

## Overview

The PDW Rust Migration project will rewrite the Personal Data Warehouse system from Python to Rust, maintaining complete functional equivalence while improving performance, memory safety, and deployment characteristics. The migration will leverage Rust's ecosystem for Excel processing, SQLite operations, and configuration management while preserving all existing data formats and workflows.

## Architecture

The Rust implementation will follow a modular architecture similar to the Python version but enhanced with Rust's ownership model and type safety:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Excel Files   │───▶│   PDW Rust       │───▶│  SQLite Database│
│  (Input Data)   │    │  (ETL Engine)    │    │   (Processed)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │   Report Files   │
                       │ (Excel/CSV/JSON) │
                       └──────────────────┘
```

### Core Modules

1. **Configuration Module** (`config.rs`) - TOML/INI configuration handling
2. **Excel Processing Module** (`excel.rs`) - Excel file reading and parsing
3. **Database Module** (`database.rs`) - SQLite operations and schema management
4. **ETL Pipeline Module** (`etl.rs`) - Data transformation and loading
5. **Reporting Module** (`reporting.rs`) - Report generation and export
6. **Error Handling Module** (`error.rs`) - Comprehensive error management
7. **Logging Module** (`logging.rs`) - Structured logging and monitoring

## Components and Interfaces

### 1. Configuration Management Component

**Purpose:** Handle TOML configuration files with backward compatibility for INI format

**Key Structures:**
```rust
#[derive(Deserialize, Debug)]
pub struct PdwConfig {
    pub directories: DirectoryConfig,
    pub file_types: FileTypeConfig,
    pub settings: SettingsConfig,
}

#[derive(Deserialize, Debug)]
pub struct DirectoryConfig {
    pub dir_in: PathBuf,
    pub dir_out: PathBuf,
    pub database_dir: PathBuf,
    pub log_dir: PathBuf,
}
```

**Interface:**
```rust
pub trait ConfigManager {
    fn load_config(path: &Path) -> Result<PdwConfig, ConfigError>;
    fn validate_config(&self) -> Result<(), ConfigError>;
    fn migrate_from_ini(ini_path: &Path) -> Result<PdwConfig, ConfigError>;
}
```

### 2. Excel Processing Component

**Purpose:** Read and parse Excel files using the `calamine` crate

**Key Structures:**
```rust
pub struct ExcelProcessor {
    workbook: Xlsx<BufReader<File>>,
}

pub struct SheetData {
    pub name: String,
    pub data: Vec<Vec<DataType>>,
    pub is_accounting: bool,
    pub is_loadable: bool,
}
```

**Interface:**
```rust
pub trait ExcelReader {
    fn open_workbook(path: &Path) -> Result<Self, ExcelError>;
    fn read_guiding_sheet(&mut self, sheet_name: &str) -> Result<Vec<SheetConfig>, ExcelError>;
    fn read_accounting_sheet(&mut self, sheet_name: &str) -> Result<Vec<Transaction>, ExcelError>;
    fn read_reference_sheet(&mut self, sheet_name: &str) -> Result<Vec<Vec<String>>, ExcelError>;
}
```

### 3. Database Management Component

**Purpose:** Handle SQLite operations using `rusqlite` crate

**Key Structures:**
```rust
pub struct DatabaseManager {
    connection: Connection,
}

pub struct Transaction {
    pub date: NaiveDate,
    pub transaction_type: String,
    pub description: String,
    pub credit: Option<f64>,
    pub debit: Option<f64>,
    pub origin: String,
}
```

**Interface:**
```rust
pub trait DatabaseOperations {
    fn create_connection(db_path: &Path) -> Result<Self, DatabaseError>;
    fn create_tables(&self) -> Result<(), DatabaseError>;
    fn insert_transactions(&self, transactions: &[Transaction]) -> Result<usize, DatabaseError>;
    fn execute_query(&self, sql: &str) -> Result<Vec<Row>, DatabaseError>;
}
```

### 4. ETL Pipeline Component

**Purpose:** Orchestrate data extraction, transformation, and loading

**Key Structures:**
```rust
pub struct EtlPipeline {
    config: PdwConfig,
    excel_processor: ExcelProcessor,
    database: DatabaseManager,
}

pub struct DataTransformer;
```

**Interface:**
```rust
pub trait EtlOperations {
    fn extract_data(&mut self) -> Result<Vec<SheetData>, EtlError>;
    fn transform_data(&self, data: Vec<SheetData>) -> Result<Vec<Transaction>, EtlError>;
    fn load_data(&self, transactions: Vec<Transaction>) -> Result<(), EtlError>;
    fn create_pivot_tables(&self) -> Result<(), EtlError>;
}
```

### 5. Reporting Component

**Purpose:** Generate reports in multiple formats using YAML query definitions

**Key Structures:**
```rust
pub struct ReportGenerator {
    database: DatabaseManager,
    queries: QueryConfig,
}

#[derive(Deserialize)]
pub struct QueryConfig {
    pub queries_gera_hist: Vec<QueryDefinition>,
    pub queries_padrao: Vec<QueryDefinition>,
}
```

**Interface:**
```rust
pub trait ReportOperations {
    fn load_queries(yaml_path: &Path) -> Result<QueryConfig, ReportError>;
    fn generate_excel_report(&self, output_path: &Path) -> Result<(), ReportError>;
    fn export_csv(&self, query: &str, output_path: &Path) -> Result<(), ReportError>;
    fn export_json(&self, query: &str, output_path: &Path) -> Result<(), ReportError>;
}
```

## Data Models

### Core Data Structures

```rust
// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: NaiveDate,
    pub day_of_week: String,
    pub transaction_type: String,
    pub description: String,
    pub credit: f64,
    pub debit: f64,
    pub month: u32,
    pub year: i32,
    pub month_name: String,
    pub year_month: String,
    pub origin: String,
}

// Configuration structures
#[derive(Deserialize, Debug)]
pub struct PdwConfig {
    pub directories: DirectoryConfig,
    pub file_types: FileTypeConfig,
    pub settings: SettingsConfig,
}

// Sheet processing configuration
#[derive(Debug)]
pub struct SheetConfig {
    pub table_name: String,
    pub is_accounting: bool,
    pub is_loadable: bool,
}

// Query definition for reports
#[derive(Deserialize, Debug)]
pub struct QueryDefinition {
    pub sql: String,
    pub sheet_name: String,
}
```

### Database Schema Mapping

The Rust implementation will maintain identical database schema to the Python version:

```sql
-- Main entries table (identical to Python version)
CREATE TABLE LANCAMENTOS_GERAIS (
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
);
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Functional Equivalence
*For any* valid Excel input file, when processed by both the Python and Rust implementations with identical configuration, the resulting SQLite databases should contain identical data structures and content
**Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**

### Property 2: Performance Improvement
*For any* processing operation (ETL, reporting, or database operations), the Rust implementation should complete the operation using less memory and in equal or less time compared to the Python implementation
**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5**

### Property 3: Cross-Platform Compatibility
*For any* supported platform (Windows, Linux, macOS), the Rust implementation should compile to a single executable and operate with identical functionality across all platforms
**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

### Property 4: Memory Safety Guarantee
*For any* execution path through the Rust code, memory safety violations (buffer overflows, use-after-free, data races) should be prevented by the Rust compiler and type system
**Validates: Requirements 4.1, 4.2, 4.3, 4.4**

### Property 5: Migration Compatibility
*For any* existing PDW data file (SQLite database, Excel template, configuration file), the Rust implementation should process it correctly either directly or through provided migration tools
**Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5**

## Error Handling

### Comprehensive Error Management

The Rust implementation will use a structured error handling approach:

```rust
#[derive(Debug, thiserror::Error)]
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
}
```

### Error Recovery Strategies

1. **Graceful Degradation:** Continue processing when non-critical errors occur
2. **Transaction Rollback:** Database operations wrapped in transactions
3. **Detailed Logging:** Comprehensive error context for troubleshooting
4. **User-Friendly Messages:** Clear error descriptions for end users

## Testing Strategy

### Unit Testing
- Test individual functions and modules with mock data
- Validate data transformation logic with known inputs/outputs
- Test error conditions and edge cases
- Use Rust's built-in testing framework with `#[cfg(test)]`

### Property-Based Testing
- Use `proptest` crate for property-based testing
- Generate random Excel data and verify processing consistency
- Test equivalence between Python and Rust implementations
- Validate performance characteristics across different data sizes

### Integration Testing
- End-to-end testing with real Excel files
- Cross-platform compilation and execution testing
- Database compatibility testing with existing Python-generated databases
- Report output validation against Python version

### Performance Testing
- Memory usage profiling during large file processing
- Execution time benchmarking against Python implementation
- Resource utilization monitoring across different platforms
- Concurrent operation safety and performance testing

## Implementation Dependencies

### Required Rust Crates

```toml
[dependencies]
# Excel file processing
calamine = "0.22"

# SQLite database operations
rusqlite = { version = "0.29", features = ["bundled"] }

# Configuration file handling
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
ini = "1.3"

# Date and time handling
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
log = "0.4"
env_logger = "0.10"

# YAML processing
serde_yaml = "0.9"

# JSON handling
serde_json = "1.0"

# Excel writing
rust_xlsxwriter = "0.49"

# CSV handling
csv = "1.2"

# Compression
flate2 = "1.0"

# Property-based testing (dev dependency)
proptest = "1.2"
```

### Development Tools

```toml
[dev-dependencies]
proptest = "1.2"
criterion = "0.5"  # For benchmarking
tempfile = "3.0"   # For testing with temporary files
```

## Migration Strategy

### Phase 1: Core Infrastructure
1. Set up Cargo project structure
2. Implement configuration management with TOML support
3. Create basic error handling framework
4. Set up logging infrastructure

### Phase 2: Data Processing
1. Implement Excel reading functionality
2. Create SQLite database operations
3. Build data transformation pipeline
4. Implement basic ETL operations

### Phase 3: Feature Parity
1. Add pivot table generation
2. Implement report generation with YAML queries
3. Add all export formats (CSV, JSON, XML)
4. Implement dynamic reports

### Phase 4: Testing and Validation
1. Create comprehensive test suite
2. Implement property-based tests for equivalence
3. Performance benchmarking against Python version
4. Cross-platform testing and validation

### Phase 5: Migration Tools
1. Create configuration migration utilities
2. Implement database compatibility verification
3. Create deployment and distribution packages
4. Documentation and migration guides

This design provides a comprehensive blueprint for migrating the PDW system to Rust while maintaining complete functional equivalence and improving performance characteristics.