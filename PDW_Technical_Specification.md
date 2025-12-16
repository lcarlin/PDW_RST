# Personal Data Warehouse (PDW) - Technical Specification

## Executive Summary

The Personal Data Warehouse (PDW) is a Python-based ETL (Extract, Transform & Load) system designed to process Excel workbooks containing financial and accounting data, transforming them into a structured SQLite database with comprehensive reporting capabilities. The system follows a modular architecture with configurable processing pipelines, dynamic report generation, and extensive data validation mechanisms.

**Version:** 9.11.0  
**Author:** Carlin, Luiz A.  
**Language:** Python 3.x  
**Database:** SQLite3  
**Primary Use Case:** Personal financial data management and reporting  

## System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Excel Files   │───▶│   PDW Engine     │───▶│  SQLite Database│
│  (Input Data)   │    │  (ETL Process)   │    │   (Processed)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │   Report Files   │
                       │ (Excel/CSV/JSON) │
                       └──────────────────┘
```

### Core Components

1. **Configuration Manager** - Handles INI file parsing and parameter validation
2. **Data Loader Engine** - Processes Excel sheets and loads data into SQLite
3. **Data Transformation Pipeline** - Sanitizes, enriches, and validates data
4. **Pivot Table Generator** - Creates aggregated views for reporting
5. **Report Generator** - Produces various output formats using YAML-defined queries
6. **Dynamic Report Engine** - Handles configurable reports based on Excel definitions

### Module Structure

```
PersonalDataWareHouse.py
├── main()                          # Entry point and orchestration
├── Configuration Management
│   ├── Config file parsing
│   └── Parameter validation
├── Data Loading (new_data_loader)
│   ├── read_guiding_sheet()
│   ├── process_accounting_sheet()
│   ├── process_non_accounting_sheet()
│   └── Data sanitization functions
├── Data Processing
│   ├── data_correjeitor()
│   ├── sanitize_entries_dataframe()
│   └── Temporal enrichment functions
├── Pivot & Aggregation
│   ├── create_pivot_history()
│   ├── monthly_summaries()
│   └── split_paymnt_resume()
├── Reporting
│   ├── xlsx_report_generator()
│   ├── general_entries_file_exportator()
│   └── create_dinamic_reports()
└── Utility Functions
    ├── gzip_compressor()
    ├── dataframe_to_xml()
    └── Helper dictionaries
```

## Detailed Component Specifications

### 1. Configuration Manager

**Purpose:** Manages system configuration through INI files with version validation and parameter extraction.

**Key Functions:**
- `main()` - Configuration loading and validation
- Version compatibility checking
- Directory and file path validation
- Boolean and string parameter extraction

**Configuration Structure:**
```ini
[DIRECTORIES]
DIR_IN = X:\Documentos\Pessoal\PDW\
DIR_OUT = X:\Documentos\Pessoal\PDW\
DATABASE_DIR = X:\Documentos\Pessoal\PDW\
LOG_DIR = X:\Documentos\Pessoal\PDW\

[FILE_TYPES]
TYPE_IN = xlsx
TYPE_OUT = xlsx
DB_FILE_TYPE = db
INPUT_FILE = PDW
OUT_DB_FILE = PDW

[SETTINGS]
CURRENT_VERSION = 9.11.0
RUN_DATA_LOADER = True
RUN_REPORTS = True
OVERWRITE_DB = True
CREATE_PIVOT = True
MULTITHREADING = False
```

### 2. Data Loader Engine

**Purpose:** Orchestrates the extraction and loading of Excel data into SQLite database.

#### Core Function: `new_data_loader()`

**Parameters:**
- `data_base` (str): SQLite database file path
- `types_sheet` (str): Name of the sheet containing transaction types
- `general_entries_table` (str): Target table for consolidated entries
- `data_origin_col` (str): Column name to track data source
- `guiding_sheet` (str): Configuration sheet name
- `excel_file` (str): Source Excel file path
- `save_useless` (bool): Whether to preserve invalid records
- `udt` (str): Discarded data table name

**Algorithm:**
1. Connect to SQLite database
2. Read guiding sheet configuration
3. Drop existing general entries table
4. Process each sheet based on configuration:
   - If accounting sheet: extract financial data
   - If non-accounting sheet: load as reference data
5. Consolidate all accounting data into single DataFrame
6. Apply data sanitization and enrichment
7. Save to database with sorting
8. Execute data correction procedures

#### Supporting Functions:

**`read_guiding_sheet(excel_file, sheet_name)`**
- Reads configuration sheet that defines processing rules
- Returns DataFrame with sheet processing instructions
- Columns: TABLE_NAME, ACCOUNTING, LOADABLE

**`process_accounting_sheet(excel_file, sheet_name, origin_col_name)`**
- Extracts financial transaction data from Excel sheet
- Standardizes column structure: Data, TIPO, DESCRICAO, Credito, Debito
- Adds origin tracking column
- Returns processed DataFrame and row count

**`process_non_accounting_sheet(excel_file, sheet_name, conn)`**
- Loads non-financial sheets directly to database
- Used for reference data and configuration tables
- Returns number of rows processed

### 3. Data Transformation Pipeline

**Purpose:** Sanitizes, validates, and enriches raw data with temporal and financial information.

#### Core Function: `sanitize_entries_dataframe()`

**Data Sanitization Steps:**
1. **Null Removal:** Removes records with null TIPO or Data fields
2. **Column Addition:** Adds temporal columns (DIA_SEMANA, Mes, Ano, MES_EXTENSO, AnoMes)
3. **Financial Sanitization:** Converts and rounds Credito/Debito to numeric with 2 decimal places
4. **Date Enrichment:** Extracts month, year, weekday information
5. **Text Cleaning:** Standardizes description text (removes special characters, standardizes separators)

#### Supporting Functions:

**`clean_description_text(text_series)`**
- Replaces semicolons and commas with pipes
- Removes special characters (∴, ś)
- Strips whitespace

**`enrich_dataframe_with_dates(df)`**
- Maps month numbers to Portuguese month names
- Maps weekday numbers to Portuguese day names
- Creates formatted date strings (YYYY/MM)

**`sanitize_financial_columns(df)`**
- Converts text to numeric values
- Handles conversion errors gracefully
- Rounds to 2 decimal places
- Fills NaN values with 0

**Temporal Dictionaries:**
```python
# Month names in Portuguese
{1: "01-Janeiro", 2: "02-Fevereiro", ...}

# Weekday names in Portuguese  
{0: "Segunda-feira", 1: "Terça-feira", ...}
```

### 4. Data Validation and Correction

**Purpose:** Ensures data integrity and removes invalid records.

#### Core Function: `data_correjeitor()`

**Validation Steps:**
1. **Useless Data Handling:** Optionally saves records with null data/tipo to separate table
2. **Null Cleanup:** Removes records with null essential fields
3. **Reference Data Validation:** Cleans type definitions table
4. **Installment Data Validation:** Validates payment installment records
5. **View Management:** Recreates database views for data origins

**SQL Operations Performed:**
```sql
-- Save invalid records (if enabled)
CREATE TABLE discarded_data AS 
SELECT * FROM entries WHERE (data IS NULL OR tipo IS NULL);

-- Remove invalid records
DELETE FROM entries WHERE (data IS NULL OR tipo IS NULL);

-- Clean reference data
DELETE FROM TiposLancamentos WHERE (Código IS NULL OR Descrição IS NULL);

-- Create origins view
CREATE VIEW Origens AS 
SELECT TABLE_NAME as nome FROM GUIDING 
WHERE LOADABLE = 'X' AND ACCOUNTING = 'X';
```

### 5. Pivot Table Generator

**Purpose:** Creates aggregated views for historical analysis and reporting.

#### Core Function: `create_pivot_history()`

**Generated Tables:**
1. **Monthly Value Pivots:** Aggregates debits by month and transaction type
2. **Monthly Count Pivots:** Counts transactions by month and transaction type  
3. **Annual Value Pivots:** Aggregates debits by year and transaction type
4. **Annual Count Pivots:** Counts transactions by year and transaction type

**Algorithm:**
1. Read transaction types from reference table
2. Read all entries from main table
3. Create pivot tables using pandas pivot_table()
4. Fill missing values with 0
5. Reorder columns based on type definitions
6. Save each pivot to separate database table

**Pivot Table Structure:**
```
Index: AnoMes (YYYY/MM) or Ano (YYYY)
Columns: Transaction types (from TiposLancamentos)
Values: Sum of Debito amounts or Count of transactions
```

### 6. Report Generator

**Purpose:** Produces various output formats using YAML-defined SQL queries.

#### Core Function: `xlsx_report_generator()`

**Features:**
- YAML-based query configuration
- Variable substitution in SQL queries
- Multiple output formats (single/multiple Excel files)
- Conditional report generation based on settings

**YAML Query Structure:**
```yaml
queries_gera_hist:
  - sql: "SELECT * FROM {full_hist} WHERE date >= date('now','-13 month');"
    sheet_name: "{full_hist}12Meses"

queries_padrao:
  - sql: "SELECT tipo, sum(debito) as Valor FROM {entries_table} GROUP BY tipo;"
    sheet_name: "Summary_Report"
```

**Variable Substitution:**
- `{entries_table}`: Main entries table name
- `{full_hist}`: Full history pivot table name
- `{anual_hist}`: Annual history pivot table name
- `{day_prog}`: Daily progress table name
- `{splt_pmnt_res}`: Split payment results table name
- `{mont_summ}`: Monthly summaries table name

#### Export Functions:

**`general_entries_file_exportator()`**
- Exports main entries table to multiple formats
- Formats: CSV (CP1252 encoding), JSON (compressed), XML (compressed)
- Applies date formatting and text transformations
- Uses custom XML generation for compatibility

**`dataframe_to_xml(df, filename)`**
- Custom XML generator for DataFrame export
- Creates structured XML with proper encoding
- Handles special characters and formatting

### 7. Dynamic Report Engine

**Purpose:** Generates configurable reports based on Excel sheet definitions.

#### Core Function: `create_dinamic_reports()`

**Process:**
1. Read dynamic report configuration from Excel sheet
2. For each report definition:
   - Read column specifications from corresponding sheet
   - Build dynamic SQL query with column summation
   - Execute query against pivot tables
   - Save results to named table

**Dynamic SQL Generation:**
```python
base_sql = "SELECT "
for column in report_columns:
    base_sql += f"HG.'{column}',"
base_sql += f"({sum_columns}) as 'Valor Total' FROM {pivot_table} HG;"
```

### 8. Utility Functions

#### File Compression: `gzip_compressor()`
- Compresses files using gzip
- Removes original file after compression
- Used for JSON and XML exports

#### Table Management: `table_droppator()`
- Safely drops database tables
- Used for cleanup operations

#### Daily Totalization: `totalizador_diario()`
- Creates cumulative daily count tables
- Tracks data loading progress over time

## Database Schema

### Core Tables

#### LANCAMENTOS_GERAIS (Main Entries Table)
```sql
CREATE TABLE LANCAMENTOS_GERAIS (
    Data DATE,                    -- Transaction date
    DIA_SEMANA TEXT,             -- Day of week (Portuguese)
    TIPO TEXT,                   -- Transaction type
    DESCRICAO TEXT,              -- Description
    Credito REAL,                -- Credit amount
    Debito REAL,                 -- Debit amount  
    Mes TEXT,                    -- Month (MM)
    Ano TEXT,                    -- Year (YYYY)
    MES_EXTENSO TEXT,            -- Month name (Portuguese)
    AnoMes TEXT,                 -- Year/Month (YYYY/MM)
    Origem TEXT                  -- Data source sheet name
);
```

#### TiposLancamentos (Transaction Types)
```sql
CREATE TABLE TiposLancamentos (
    Código TEXT,                 -- Type code
    Descrição TEXT               -- Type description
);
```

#### GUIDING (Processing Configuration)
```sql
CREATE TABLE GUIDING (
    TABLE_NAME TEXT,             -- Excel sheet name
    ACCOUNTING TEXT,             -- 'X' if accounting sheet
    LOADABLE TEXT                -- 'X' if should be processed
);
```

### Pivot Tables

#### HistoricoGeral (Monthly Pivot)
```sql
CREATE TABLE HistoricoGeral (
    AnoMes TEXT,                 -- YYYY/MM
    [Transaction_Type_1] REAL,   -- Dynamic columns based on
    [Transaction_Type_2] REAL,   -- transaction types
    ...
);
```

#### HistoricoAnual (Annual Pivot)
```sql
CREATE TABLE HistoricoAnual (
    Ano TEXT,                    -- YYYY
    [Transaction_Type_1] REAL,   -- Dynamic columns based on
    [Transaction_Type_2] REAL,   -- transaction types
    ...
);
```

### Summary Tables

#### Resumido_In_Out (Monthly Summaries)
```sql
CREATE TABLE Resumido_In_Out (
    AnoMes TEXT,                 -- YYYY/MM
    Origem TEXT,                 -- Data source
    CREDITO REAL,                -- Total credits
    DEBITO REAL,                 -- Total debits
    Posição REAL                 -- Net position (credits - debits)
);
```

#### Resumo_Parcelamentos (Installment Summary)
```sql
CREATE TABLE Resumo_Parcelamentos (
    Ano_Mes TEXT,                -- YYYY-MM
    Quantidade INTEGER,          -- Count of installments
    Valor REAL,                  -- Total value
    Diff_QTD INTEGER,            -- Change in quantity
    Diff_Vlr REAL                -- Change in value
);
```

## Error Handling

### Configuration Errors
- **FileNotFoundError:** Configuration file missing
- **ConfigParser.Error:** Invalid INI format
- **Version Mismatch:** Parameter file version != code version

### Data Processing Errors
- **File Access Errors:** Input/output directory validation
- **Excel Processing Errors:** Sheet reading failures
- **Database Errors:** SQLite connection and operation failures
- **Data Validation Errors:** Invalid data type conversions

### Recovery Mechanisms
- Graceful degradation for missing optional features
- Detailed error logging with timestamps
- Transaction rollback for database operations
- Validation of critical paths before processing

## Performance Considerations

### Memory Management
- Processes Excel sheets individually to limit memory usage
- Uses pandas chunking for large datasets
- Explicit DataFrame cleanup after processing

### Database Optimization
- Drops and recreates tables for clean state
- Uses batch operations for data insertion
- Implements proper indexing (commented out for flexibility)

### Threading Considerations
- Explicitly disables multithreading due to SQLite limitations
- Provides educational commentary on threading risks
- Single-threaded design ensures data consistency

## Dependencies and Requirements

### Python Packages
```
pandas>=1.0.0          # Data manipulation and analysis
xlsxwriter>=1.0.0      # Excel file writing
xlrd>=1.0.0            # Excel file reading (legacy)
openpyxl>=3.0.0        # Excel file reading/writing
sqlalchemy>=1.3.0      # Database toolkit
numpy>=1.18.0          # Numerical computing
configparser           # Configuration file parsing (built-in)
sqlite3                # SQLite database (built-in)
datetime               # Date/time handling (built-in)
xml.etree.ElementTree  # XML processing (built-in)
gzip                   # File compression (built-in)
shutil                 # File operations (built-in)
yaml>=5.0.0            # YAML file processing
```

### System Requirements
- Python 3.6+
- Windows/Linux/macOS
- Minimum 512MB RAM
- 100MB disk space for database and reports

### File System Requirements
- Read access to input Excel files
- Write access to output directories
- SQLite database file permissions
- Log file write permissions

This technical specification provides the foundation for understanding and re-implementing the Personal Data Warehouse system in any programming language, with complete algorithmic details and architectural insights.