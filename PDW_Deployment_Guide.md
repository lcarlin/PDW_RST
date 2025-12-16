# Personal Data Warehouse (PDW) - Deployment and Operations Guide

## System Requirements

### Hardware Requirements
- **Minimum RAM:** 512 MB (2 GB recommended for large datasets)
- **Disk Space:** 100 MB for application + storage for databases and reports
- **Processor:** Any modern CPU (single-threaded application)
- **Network:** Not required (standalone application)

### Operating System Support
- **Windows:** Windows 7/8/10/11 (32-bit or 64-bit)
- **Linux:** Any modern distribution with Python 3.6+
- **macOS:** macOS 10.12+ with Python 3.6+

### Python Environment Requirements
- **Python Version:** 3.6 or higher (3.8+ recommended)
- **Package Manager:** pip (included with Python)
- **Virtual Environment:** Recommended but not required

## Installation Instructions

### Step 1: Python Installation

**Windows:**
1. Download Python from https://python.org/downloads/
2. Run installer with "Add Python to PATH" option checked
3. Verify installation: `python --version`

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install python3 python3-pip
```

**Linux (CentOS/RHEL):**
```bash
sudo yum install python3 python3-pip
```

**macOS:**
```bash
# Using Homebrew
brew install python3

# Or download from python.org
```

### Step 2: Dependency Installation

**Required Python Packages:**
```bash
pip install pandas>=1.0.0
pip install xlsxwriter>=1.0.0
pip install xlrd>=1.0.0
pip install openpyxl>=3.0.0
pip install sqlalchemy>=1.3.0
pip install numpy>=1.18.0
pip install PyYAML>=5.0.0
```

**Alternative: Requirements File Installation**
Create `requirements.txt`:
```
pandas>=1.0.0
xlsxwriter>=1.0.0
xlrd>=1.0.0
openpyxl>=3.0.0
sqlalchemy>=1.3.0
numpy>=1.18.0
PyYAML>=5.0.0
```

Install all at once:
```bash
pip install -r requirements.txt
```

### Step 3: Application Setup

**File Structure:**
```
PDW_Installation/
├── PersonalDataWareHouse.py      # Main application
├── PersonalDataWareHouse.cfg     # Configuration file
├── PDW_QUERIES.yaml              # Report queries
├── input/                        # Input Excel files
├── output/                       # Generated reports
├── database/                     # SQLite databases
└── logs/                         # System logs
```

**Directory Creation:**
```bash
mkdir PDW_Installation
cd PDW_Installation
mkdir input output database logs
```

## Configuration Setup

### Configuration File (PersonalDataWareHouse.cfg)

**Template Configuration:**
```ini
[DIRECTORIES]
DIR_IN = ./input/
DIR_OUT = ./output/
DATABASE_DIR = ./database/
LOG_DIR = ./logs/

[FILE_TYPES]
TYPE_IN = xlsx
TYPE_OUT = xlsx
DB_FILE_TYPE = db
LOG_FILE = PDW.SysMap.log
INPUT_FILE = PDW
OUT_DB_FILE = PDW
OUT_RPT_FILE = PDW_REPORTS.v2

[SETTINGS]
CURRENT_VERSION = 9.11.0
RUN_DATA_LOADER = True
RUN_REPORTS = True
OVERWRITE_DB = True
CREATE_PIVOT = True
RPT_SINGLE_FILE = True
MULTITHREADING = False
SAVE_DISCARTED_DATA = False
EXPORT_OTHER_TYPES = False
RUN_DINAMIC_REPORT = True
```

### YAML Queries File (PDW_QUERIES.yaml)

**Basic Query Configuration:**
```yaml
queries_padrao:
  - sql: >
      SELECT tipo as Categoria, sum(debito) as Valor, count(1) as QTD 
      FROM {entries_table}
      WHERE Data >= date('now','-1 month')  
      GROUP BY tipo 
      ORDER BY 2 DESC;
    sheet_name: "Ultimos30Dias"
```

## Excel File Preparation

### Required Excel Structure

**1. Guiding Sheet (GUIDING):**
```
TABLE_NAME          | ACCOUNTING | LOADABLE
--------------------|------------|----------
ContaCorrente      | X          | X
CartaoCredito      | X          | X
TiposLancamentos   |            | X
Parcelamentos      | X          | X
```

**2. Transaction Types Sheet (TiposLancamentos):**
```
Código | Descrição
-------|------------------
ALM    | Alimentação
TRP    | Transporte
SAU    | Saúde
EDU    | Educação
```

**3. Accounting Sheets (e.g., ContaCorrente):**
```
Data       | TIPO | DESCRICAO           | Credito | Debito
-----------|------|---------------------|---------|--------
2024-01-15 | ALM  | Supermercado XYZ    |         | 150.50
2024-01-16 | SAL  | Salário Janeiro     | 3000.00 |
```

## Deployment Procedures

### Development Environment Setup

**1. Clone/Download Application:**
```bash
# If using version control
git clone <repository_url>

# Or download and extract files
```

**2. Virtual Environment (Recommended):**
```bash
python -m venv pdw_env

# Windows
pdw_env\Scripts\activate

# Linux/macOS
source pdw_env/bin/activate
```

**3. Install Dependencies:**
```bash
pip install -r requirements.txt
```

**4. Configure Application:**
- Edit `PersonalDataWareHouse.cfg` with correct paths
- Customize `PDW_QUERIES.yaml` for specific reports
- Prepare Excel input file with required structure

### Production Deployment

**1. Standalone Executable (Optional):**
```bash
pip install pyinstaller
pyinstaller -F PersonalDataWareHouse.py
```

**2. Scheduled Execution:**

**Windows Task Scheduler:**
- Create basic task
- Set trigger (daily/weekly/monthly)
- Action: Start program `python.exe`
- Arguments: `C:\path\to\PersonalDataWareHouse.py`
- Start in: `C:\path\to\PDW_Installation\`

**Linux Cron:**
```bash
# Edit crontab
crontab -e

# Add entry for daily execution at 6 AM
0 6 * * * cd /path/to/PDW_Installation && python PersonalDataWareHouse.py
```

## File System Permissions

### Required Permissions

**Input Directory:**
- Read access to Excel files
- Directory listing permissions

**Output Directory:**
- Write access for report files
- Create/delete file permissions

**Database Directory:**
- Read/write access for SQLite files
- File locking permissions (for SQLite)

**Log Directory:**
- Write access for log files
- Append permissions for existing logs

### Security Considerations

**File Access:**
- Ensure Excel files are not password protected
- Close Excel files before processing
- Backup important data before processing

**Database Security:**
- SQLite files contain sensitive financial data
- Consider encryption for sensitive environments
- Regular backups of database files

## Monitoring and Maintenance

### Log File Management

**Log File Location:** Configured in `LOG_DIR` setting

**Log Entry Format:**
```
2024/01/15 08:30:15 Started | 2024/01/15 08:32:45 Ended | 150.23 TotalSecs | Version 9.11.0 | Hostname DESKTOP-ABC | OS Windows
```

**Log Monitoring:**
- Check for error messages in logs
- Monitor execution times for performance issues
- Verify successful completion of all phases

### Database Maintenance

**Regular Tasks:**
1. **Backup Database Files:** Copy .db files to secure location
2. **Monitor Database Size:** Check disk space usage
3. **Verify Data Integrity:** Spot-check report outputs
4. **Clean Old Databases:** Remove outdated database versions if not using overwrite mode

### Performance Monitoring

**Key Metrics:**
- **Execution Time:** Total processing time per run
- **Memory Usage:** Peak memory consumption during processing
- **Data Volume:** Number of records processed
- **Error Rate:** Frequency of processing errors

**Performance Optimization:**
- Regular cleanup of temporary files
- Optimize Excel file structure (remove unused sheets)
- Monitor system resources during execution
- Consider data archiving for very large datasets

## Troubleshooting Guide

### Common Issues and Solutions

**1. Configuration File Not Found**
```
Error: Configuration file PersonalDataWareHouse.cfg not found!
Solution: Ensure config file exists in same directory as Python script
```

**2. Version Mismatch**
```
Error: The version in parameter file does not Match
Solution: Update CURRENT_VERSION in config file to match application version (9.11.0)
```

**3. Input Directory Not Found**
```
Error: The Input Directory does not exists
Solution: Create directory specified in DIR_IN setting or update path in config
```

**4. Excel File Access Error**
```
Error: Input Load File does not exists
Solution: Verify Excel file name matches INPUT_FILE setting and file exists in DIR_IN
```

**5. Permission Denied Errors**
```
Error: Permission denied writing to output directory
Solution: Check file/directory permissions, ensure Excel files are closed
```

**6. Memory Errors with Large Files**
```
Error: MemoryError during Excel processing
Solution: Close other applications, increase system RAM, or split large Excel files
```

**7. Database Lock Errors**
```
Error: Database is locked
Solution: Ensure no other applications are accessing SQLite file, restart if necessary
```

### Diagnostic Steps

**1. Verify Installation:**
```bash
python --version
pip list | grep pandas
python -c "import pandas; print(pandas.__version__)"
```

**2. Test Configuration:**
```bash
python PersonalDataWareHouse.py --help  # If help option implemented
```

**3. Check File Permissions:**
```bash
# Linux/macOS
ls -la input/ output/ database/ logs/

# Windows
dir input output database logs
```

**4. Validate Excel File:**
- Open Excel file manually to verify structure
- Check for password protection
- Verify sheet names match configuration

### Recovery Procedures

**1. Corrupted Database Recovery:**
- Restore from backup if available
- Delete corrupted .db file and re-run processing
- Check disk space and file system integrity

**2. Configuration Reset:**
- Restore default configuration file
- Verify all directory paths exist
- Reset version number to current application version

**3. Data Recovery:**
- Keep backup copies of original Excel files
- Use database export features to recover processed data
- Implement regular backup procedures

## Version Management and Upgrades

### Version Compatibility

**Current Version:** 9.11.0
**Configuration Compatibility:** Must match exactly
**Database Compatibility:** Forward compatible (newer versions can read older databases)

### Upgrade Procedures

**1. Backup Current Installation:**
- Copy all configuration files
- Backup database files
- Save custom YAML query files

**2. Install New Version:**
- Download new application files
- Update Python dependencies if required
- Update configuration file version number

**3. Test Upgrade:**
- Run with test data first
- Verify all reports generate correctly
- Check log files for any warnings

**4. Migration Considerations:**
- Review changelog for breaking changes
- Update custom queries if SQL syntax changes
- Test all custom reports and exports

This deployment guide provides comprehensive instructions for setting up, configuring, and maintaining the Personal Data Warehouse system in various environments.