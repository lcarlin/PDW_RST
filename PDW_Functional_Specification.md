# Personal Data Warehouse (PDW) - Functional Specification

## Executive Summary

The Personal Data Warehouse (PDW) is a comprehensive financial data management system designed for personal accounting and financial analysis. It automates the process of extracting financial data from Excel spreadsheets, transforming it into a structured format, and generating detailed reports for financial planning and analysis.

## Business Context and Purpose

### Primary Business Objectives
1. **Data Consolidation:** Centralize financial data from multiple Excel sources into a single, queryable database
2. **Automated Processing:** Eliminate manual data entry and reduce human error in financial record keeping
3. **Historical Analysis:** Provide trend analysis and historical comparisons for financial planning
4. **Flexible Reporting:** Generate customizable reports in multiple formats for different analytical needs
5. **Data Integrity:** Ensure accuracy and consistency of financial records through automated validation

### Target Users
- **Primary User:** Individual managing personal finances with complex Excel-based tracking systems
- **Secondary Users:** Small business owners, freelancers, or financial advisors managing client data
- **Technical Users:** Developers or analysts who need to extend or customize the system

## Business Processes and Workflows

### 1. Data Input and Preparation Workflow

**Business Process:** Users maintain financial records in Excel workbooks with multiple sheets representing different accounts, transaction types, or time periods.

**Input Requirements:**
- **Master Excel Workbook:** Contains multiple sheets with financial data
- **Guiding Sheet:** Configuration sheet that defines which sheets to process and how
- **Transaction Types Sheet:** Reference data defining valid transaction categories
- **Accounting Sheets:** Contain actual financial transactions with standardized columns
- **Reference Sheets:** Non-transactional data like account definitions or categories

**Data Structure Requirements:**
```
Required Columns for Accounting Sheets:
- Data: Transaction date (Excel date format)
- TIPO: Transaction type (must match reference data)
- DESCRICAO: Transaction description (free text)
- Credito: Credit amount (numeric, can be empty)
- Debito: Debit amount (numeric, can be empty)
```

### 2. ETL (Extract, Transform, Load) Workflow

**Business Logic:**

**Step 1: Configuration Validation**
- System validates configuration file version matches application version
- Verifies all required directories exist and are accessible
- Confirms input Excel file exists and is readable

**Step 2: Data Extraction**
- Reads guiding sheet to determine processing rules for each Excel sheet
- Extracts data from sheets marked as "accounting" sheets
- Loads reference data from non-accounting sheets directly to database
- Tracks data origin for audit trail purposes

**Step 3: Data Transformation**
- **Date Enrichment:** Adds day of week, month name, year, and year/month combinations
- **Financial Standardization:** Converts text to numeric, rounds to 2 decimal places
- **Text Cleaning:** Standardizes description text, removes special characters
- **Validation:** Removes records with missing essential data (date or transaction type)
- **Origin Tracking:** Adds source sheet name to each record for traceability

**Step 4: Data Loading**
- Consolidates all accounting data into single main table
- Sorts data by date (most recent first) for optimal query performance
- Applies additional data quality checks and corrections
- Creates database views for commonly accessed data subsets
### 3. Data Analysis and Aggregation Workflow

**Business Process:** System creates multiple analytical views of the data to support different types of financial analysis.

**Pivot Table Generation:**
- **Monthly Analysis:** Aggregates transactions by month and type for trend analysis
- **Annual Analysis:** Summarizes yearly totals by transaction type for long-term planning
- **Transaction Counting:** Tracks volume of transactions (not just amounts) for activity analysis
- **Cross-tabulation:** Creates matrix views showing transaction types vs. time periods

**Business Rules for Aggregation:**
1. **Debit Focus:** Primary aggregations use debit amounts (expenses/outflows)
2. **Type Consistency:** All aggregations maintain transaction type categories from reference data
3. **Time Standardization:** Uses YYYY/MM format for consistent monthly grouping
4. **Zero Filling:** Missing data points filled with zero for complete time series

### 4. Reporting and Export Workflow

**Business Process:** System generates multiple report formats to serve different analytical and operational needs.

**Report Categories:**

**1. Operational Reports:**
- **General Entries Export:** Complete transaction listing with formatted dates and amounts
- **Daily Progress Tracking:** Cumulative count of data entries over time
- **Data Quality Reports:** Summary of processing results and any data issues

**2. Analytical Reports:**
- **Monthly Summaries:** Income vs. expenses by source and time period
- **Transaction Analysis:** Breakdown by type, frequency, and amounts
- **Trend Reports:** Historical comparisons and growth analysis
- **Installment Tracking:** Special handling for payment plans and recurring transactions

**3. Dynamic Reports:**
- **Configurable Analysis:** User-defined reports based on Excel sheet specifications
- **Custom Aggregations:** Flexible grouping and summarization based on business needs
- **Ad-hoc Queries:** YAML-defined SQL queries for specific analytical requirements

**Output Formats:**
- **Excel Files:** Single or multiple sheet workbooks for detailed analysis
- **CSV Files:** Comma-separated format for integration with other tools
- **JSON Files:** Structured data format for web applications or APIs
- **XML Files:** Standardized format for data exchange (with compression)

### 5. Configuration Management Workflow

**Business Process:** System behavior is controlled through configuration files that allow customization without code changes.

**Configuration Categories:**

**1. Directory Management:**
- Input directory for Excel files
- Output directory for reports and exports
- Database directory for SQLite files
- Log directory for system monitoring

**2. Processing Control:**
- Enable/disable data loading phase
- Enable/disable report generation phase
- Control pivot table creation
- Manage dynamic report processing

**3. Data Quality Settings:**
- Option to save discarded/invalid records for review
- Control over data validation strictness
- Configuration of text cleaning rules

**4. Performance Settings:**
- Database overwrite vs. versioning options
- Single vs. multiple output file generation
- Compression settings for large exports

## Business Rules and Validation

### Data Validation Rules

**1. Essential Field Validation:**
- Transaction date must be present and valid
- Transaction type must match reference data
- At least one of credit or debit amount must be non-zero

**2. Financial Amount Rules:**
- All amounts rounded to 2 decimal places for consistency
- Negative amounts not allowed (use appropriate debit/credit classification)
- Zero amounts preserved but flagged for review

**3. Text Standardization Rules:**
- Description text limited to standard character set
- Special characters replaced with standard alternatives
- Consistent separator usage (pipes instead of commas/semicolons)

**4. Temporal Consistency Rules:**
- All dates converted to standard format for comparison
- Month/year derivations must be consistent with source date
- Day of week calculations verified against date

### Business Logic Rules

**1. Transaction Classification:**
- Each transaction must have exactly one type classification
- Types must be predefined in reference data
- New types require manual addition to reference sheet

**2. Source Tracking:**
- Every transaction tagged with originating Excel sheet name
- Source information preserved through all transformations
- Audit trail maintained for data lineage

**3. Aggregation Rules:**
- Monthly aggregations use calendar months (1st to last day)
- Annual aggregations use calendar years (January to December)
- Partial periods handled consistently (no pro-rating)

**4. Reporting Rules:**
- All monetary amounts formatted consistently across reports
- Date formats standardized for user readability
- Null values handled gracefully in all outputs

## User Interactions and Interfaces

### Command Line Interface

**Primary Interaction Method:** System operates as command-line application with optional configuration file parameter.

**Usage Patterns:**
```bash
# Standard execution with default configuration
python PersonalDataWareHouse.py

# Execution with custom configuration file
python PersonalDataWareHouse.py custom_config.cfg
```

**User Feedback:**
- Color-coded console output for different message types
- Progress indicators for long-running operations
- Detailed logging with timestamps and operation results
- Error messages with specific guidance for resolution

### Configuration File Interface

**User Customization:** All system behavior controlled through INI-format configuration file.

**Key User Decisions:**
1. **Processing Scope:** Which phases to execute (loading, pivot creation, reporting)
2. **Output Preferences:** Single vs. multiple files, compression options
3. **Data Quality:** Strictness of validation, handling of invalid records
4. **Performance:** Database management options, threading preferences

### Excel Interface Requirements

**User Responsibilities:**
1. **Data Structure:** Maintain consistent column structure in accounting sheets
2. **Reference Data:** Keep transaction types and guiding sheet updated
3. **File Management:** Ensure Excel file is closed during processing
4. **Data Quality:** Review and clean data before processing when possible

## Integration Points and Data Exchange

### Input Integration

**Excel File Requirements:**
- Must be .xlsx format (modern Excel format)
- Multiple sheets supported with different purposes
- Consistent column naming within accounting sheets
- Reference data sheets for configuration and validation

**Configuration Integration:**
- INI file format for easy editing
- YAML file format for complex query definitions
- Version synchronization between configuration and application

### Output Integration

**Database Integration:**
- SQLite format for broad compatibility
- Standard SQL interface for external tool access
- Structured table design for easy querying

**File Export Integration:**
- Multiple format support (Excel, CSV, JSON, XML)
- Standardized encoding (UTF-8 for JSON/XML, CP1252 for CSV)
- Compression options for large datasets

### External System Integration

**Potential Integration Points:**
- Accounting software import (via CSV/Excel export)
- Business intelligence tools (via SQLite database)
- Web applications (via JSON export)
- Backup systems (via compressed file exports)

## Performance and Scalability Considerations

### Data Volume Handling

**Current Capacity:**
- Designed for personal/small business scale (thousands to tens of thousands of transactions)
- Memory-efficient processing through pandas DataFrame operations
- Single-threaded design ensures data consistency

**Scalability Limitations:**
- Excel file size limited by available system memory
- SQLite database size practically unlimited for target use case
- Processing time scales linearly with data volume

### Processing Performance

**Optimization Strategies:**
- Batch processing of Excel sheets
- Efficient DataFrame operations for data transformation
- Database table recreation for optimal performance
- Selective processing based on configuration flags

**Performance Monitoring:**
- Execution time tracking and reporting
- Memory usage optimization through explicit cleanup
- Progress indicators for user feedback during long operations

This functional specification provides a complete business-oriented view of the Personal Data Warehouse system, enabling users and stakeholders to understand its capabilities, limitations, and proper usage patterns.