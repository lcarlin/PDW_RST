/*!
# ETL Pipeline Module

Orchestrates the Extract, Transform, Load process for Excel to SQLite conversion.
Handles data transformation, enrichment, and validation.
*/

use crate::config::PdwConfig;
use crate::database::{DatabaseManager, ProcessedTransaction};
use crate::error::{EtlError, PdwError};
use crate::excel::{ExcelProcessor, Transaction, SheetConfig};
use crate::logging;
use chrono::{NaiveDate, Datelike, Weekday};
use std::collections::HashMap;

/// ETL Pipeline orchestrator
pub struct EtlPipeline {
    config: PdwConfig,
    database: DatabaseManager,
}

impl EtlPipeline {
    /// Create new ETL pipeline
    pub fn new(config: PdwConfig) -> Result<Self, PdwError> {
        let db_path = config.get_database_path();
        let database = DatabaseManager::new(&db_path)?;
        
        Ok(Self { config, database })
    }
    
    /// Get configuration reference
    pub fn config(&self) -> &PdwConfig {
        &self.config
    }
    
    /// Execute data loading phase
    pub fn execute_data_loading(&mut self) -> Result<(), PdwError> {
        logging::log_phase_start("Running Loader of the Sheets into database Tables");
        
        // Create database tables
        self.database.create_tables()?;
        
        // Drop existing general entries table
        self.database.drop_table(&self.config.settings.general_entries_table)?;
        
        // Open Excel file
        let input_file = self.config.get_input_file_path();
        let mut excel_processor = ExcelProcessor::new(&input_file)?;
        
        // Read guiding sheet configuration
        let sheet_configs = excel_processor.read_guiding_sheet(&self.config.settings.guiding_table)?;
        
        // Process each sheet according to configuration
        let mut all_transactions = Vec::new();
        let mut step_counter = 1;
        
        for config in &sheet_configs {
            logging::log_step(
                step_counter,
                &format!("Table (Sheet) :-> {}", config.table_name.trim()),
                ""
            );
            
            if config.is_loadable {
                if config.is_accounting {
                    // Process accounting sheet
                    let transactions = excel_processor.read_accounting_sheet(&config.table_name)?;
                    logging::log_result("Lines Created", transactions.len());
                    all_transactions.extend(transactions);
                } else {
                    // Process reference sheet
                    let data = excel_processor.read_reference_sheet(&config.table_name)?;
                    let count = self.database.insert_reference_data(&config.table_name, &data)?;
                    logging::log_result("Lines Created", count);
                }
            } else {
                logging::log_result("Skipped", 0);
            }
            
            step_counter += 1;
        }
        
        // Transform and enrich transaction data
        let processed_transactions = self.transform_transactions(all_transactions)?;
        
        // Insert processed transactions
        let count = self.database.insert_transactions(&processed_transactions)?;
        logging::log_result("Total Transactions Processed", count);
        
        // Perform data validation and cleanup
        self.database.validate_and_clean_data(
            &self.config.settings.general_entries_table,
            &self.config.settings.types_of_entries,
            self.config.settings.save_discarted_data,
            &self.config.settings.discarted_data_table,
        )?;
        
        Ok(())
    }
    
    /// Transform raw transactions into processed format
    fn transform_transactions(&self, transactions: Vec<Transaction>) -> Result<Vec<ProcessedTransaction>, PdwError> {
        let mut processed = Vec::new();
        
        for transaction in transactions {
            if let Some(processed_transaction) = self.process_single_transaction(transaction)? {
                processed.push(processed_transaction);
            }
        }
        
        // Sort by date (most recent first)
        processed.sort_by(|a, b| b.date.cmp(&a.date));
        
        Ok(processed)
    }
    
    /// Process a single transaction with data enrichment
    fn process_single_transaction(&self, transaction: Transaction) -> Result<Option<ProcessedTransaction>, PdwError> {
        // Skip transactions without essential data
        let date = match transaction.date {
            Some(d) => d,
            None => return Ok(None),
        };
        
        let transaction_type = match transaction.transaction_type {
            Some(t) => t.trim().to_string(),
            None => return Ok(None),
        };
        
        if transaction_type.is_empty() {
            return Ok(None);
        }
        
        // Clean and process description
        let description = transaction.description
            .unwrap_or_else(|| "".to_string())
            .trim()
            .replace(";", "|")
            .replace(",", "|")
            .replace("∴", " .'. ")
            .replace("ś", "s");
        
        // Process financial amounts
        let credit = transaction.credit.unwrap_or(0.0);
        let debit = transaction.debit.unwrap_or(0.0);
        
        // Round to 2 decimal places
        let credit = (credit * 100.0).round() / 100.0;
        let debit = (debit * 100.0).round() / 100.0;
        
        // Generate temporal data
        let day_of_week = self.get_day_of_week_portuguese(date);
        let month = format!("{:02}", date.month());
        let year = date.year().to_string();
        let month_name = self.get_month_name_portuguese(date.month());
        let year_month = format!("{}/{:02}", date.year(), date.month());
        
        Ok(Some(ProcessedTransaction {
            date,
            day_of_week,
            transaction_type,
            description,
            credit,
            debit,
            month,
            year,
            month_name,
            year_month,
            origin: transaction.origin,
        }))
    }
    
    /// Get Portuguese day of week name
    fn get_day_of_week_portuguese(&self, date: NaiveDate) -> String {
        match date.weekday() {
            Weekday::Mon => "Segunda-feira",
            Weekday::Tue => "Terça-feira", 
            Weekday::Wed => "Quarta-feira",
            Weekday::Thu => "Quinta-feira",
            Weekday::Fri => "Sexta-feira",
            Weekday::Sat => "Sábado",
            Weekday::Sun => "Domingo",
        }.to_string()
    }
    
    /// Get Portuguese month name
    fn get_month_name_portuguese(&self, month: u32) -> String {
        match month {
            1 => "01-Janeiro",
            2 => "02-Fevereiro",
            3 => "03-Março",
            4 => "04-Abril",
            5 => "05-Maio",
            6 => "06-Junho",
            7 => "07-Julho",
            8 => "08-Agosto",
            9 => "09-Setembro",
            10 => "10-Outubro",
            11 => "11-Novembro",
            12 => "12-Dezembro",
            _ => "00-Inválido",
        }.to_string()
    }
    
    /// Create pivot tables for historical analysis
    pub fn create_pivot_tables(&self) -> Result<(), PdwError> {
        logging::log_phase_start("Creating pivot Tables");
        
        self.database.create_pivot_tables(
            &self.config.settings.general_entries_table,
            &self.config.settings.types_of_entries,
            &self.config.settings.full_pivot_table,
            &self.config.settings.anual_pivot_table,
        )?;
        
        Ok(())
    }
    
    /// Generate reports
    pub fn generate_reports(&self) -> Result<(), PdwError> {
        logging::log_phase_start("Starting report generation");
        
        // Create daily progress tracking
        self.create_daily_progress()?;
        
        // Create monthly summaries
        self.create_monthly_summaries()?;
        
        // Create installment summaries
        self.create_installment_summaries()?;
        
        // Generate Excel reports
        self.generate_excel_reports()?;
        
        // Export general entries
        self.export_general_entries()?;
        
        Ok(())
    }
    
    /// Create daily progress tracking
    fn create_daily_progress(&self) -> Result<(), PdwError> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} AS
             SELECT Data, COUNT(*) as Contagem,
                    SUM(COUNT(*)) OVER (ORDER BY Data) as 'Contagem Acumulada'
             FROM {} 
             GROUP BY Data 
             ORDER BY Data DESC",
            self.config.settings.dayly_progress,
            self.config.settings.general_entries_table
        );
        
        self.database.connection().execute(&query, [])
            .map_err(|e| EtlError::TransformationFailed {
                stage: "daily_progress".to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Create monthly summaries
    fn create_monthly_summaries(&self) -> Result<(), PdwError> {
        let base_table = &self.config.settings.monthly_summaties;
        
        // Monthly summaries
        let monthly_query = format!(
            "CREATE TABLE IF NOT EXISTS {} AS
             SELECT AnoMes, Origem, 
                    SUM(Credito) as CREDITO,
                    SUM(Debito) as DEBITO,
                    (SUM(Credito) - SUM(Debito)) as Posição
             FROM {} 
             GROUP BY AnoMes, Origem 
             ORDER BY Origem, AnoMes",
            base_table,
            self.config.settings.general_entries_table
        );
        
        self.database.connection().execute(&monthly_query, [])
            .map_err(|e| EtlError::TransformationFailed {
                stage: "monthly_summaries".to_string(),
                reason: e.to_string(),
            })?;
        
        // Annual summaries
        let annual_query = format!(
            "CREATE TABLE IF NOT EXISTS {}_ANUAL AS
             SELECT Ano, Origem,
                    SUM(Credito) as CREDITO,
                    SUM(Debito) as DEBITO,
                    (SUM(Credito) - SUM(Debito)) as Posição
             FROM {} 
             GROUP BY Ano, Origem 
             ORDER BY Origem, Ano",
            base_table,
            self.config.settings.general_entries_table
        );
        
        self.database.connection().execute(&annual_query, [])
            .map_err(|e| EtlError::TransformationFailed {
                stage: "annual_summaries".to_string(),
                reason: e.to_string(),
            })?;
        
        // Full summaries
        let full_query = format!(
            "CREATE TABLE IF NOT EXISTS {}_FULL AS
             SELECT Origem,
                    SUM(Credito) as CREDITO,
                    SUM(Debito) as DEBITO,
                    (SUM(Credito) - SUM(Debito)) as Posição
             FROM {} 
             GROUP BY Origem 
             ORDER BY Origem",
            base_table,
            self.config.settings.general_entries_table
        );
        
        self.database.connection().execute(&full_query, [])
            .map_err(|e| EtlError::TransformationFailed {
                stage: "full_summaries".to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Create installment summaries
    fn create_installment_summaries(&self) -> Result<(), PdwError> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} AS
             SELECT strftime('%Y-%m', Data) as Ano_Mes,
                    COUNT(*) as Quantidade,
                    ROUND(SUM(Debito), 2) as Valor,
                    0 as Diff_QTD,
                    0.0 as Diff_Vlr
             FROM {}
             GROUP BY strftime('%Y-%m', Data)
             ORDER BY Ano_Mes DESC",
            self.config.settings.out_res_pmnt_tab,
            self.config.settings.splt_paymnt_tab
        );
        
        self.database.connection().execute(&query, [])
            .map_err(|e| EtlError::TransformationFailed {
                stage: "installment_summaries".to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(())
    }
    
    /// Generate Excel reports (placeholder - will be implemented in reporting module)
    fn generate_excel_reports(&self) -> Result<(), PdwError> {
        // This will be implemented in the reporting module
        log::info!("Excel report generation will be implemented in reporting module");
        Ok(())
    }
    
    /// Export general entries (placeholder - will be implemented in reporting module)
    fn export_general_entries(&self) -> Result<(), PdwError> {
        // This will be implemented in the reporting module
        log::info!("General entries export will be implemented in reporting module");
        Ok(())
    }
}

/// Trait for ETL operations
pub trait EtlOperations {
    fn extract_data(&mut self) -> Result<Vec<Transaction>, PdwError>;
    fn transform_data(&self, data: Vec<Transaction>) -> Result<Vec<ProcessedTransaction>, PdwError>;
    fn load_data(&self, transactions: Vec<ProcessedTransaction>) -> Result<(), PdwError>;
    fn create_pivot_tables(&self) -> Result<(), PdwError>;
}

impl EtlOperations for EtlPipeline {
    fn extract_data(&mut self) -> Result<Vec<Transaction>, PdwError> {
        let input_file = self.config.get_input_file_path();
        let mut excel_processor = ExcelProcessor::new(&input_file)?;
        
        let sheet_configs = excel_processor.read_guiding_sheet(&self.config.settings.guiding_table)?;
        let mut all_transactions = Vec::new();
        
        for config in &sheet_configs {
            if config.is_loadable && config.is_accounting {
                let transactions = excel_processor.read_accounting_sheet(&config.table_name)?;
                all_transactions.extend(transactions);
            }
        }
        
        Ok(all_transactions)
    }
    
    fn transform_data(&self, data: Vec<Transaction>) -> Result<Vec<ProcessedTransaction>, PdwError> {
        self.transform_transactions(data)
    }
    
    fn load_data(&self, transactions: Vec<ProcessedTransaction>) -> Result<(), PdwError> {
        self.database.insert_transactions(&transactions)?;
        Ok(())
    }
    
    fn create_pivot_tables(&self) -> Result<(), PdwError> {
        self.create_pivot_tables()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::NaiveDate;
    
    #[test]
    fn test_day_of_week_portuguese() {
        let config = PdwConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = DatabaseManager::new(&db_path).unwrap();
        
        let pipeline = EtlPipeline { config, database };
        
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); // Monday
        assert_eq!(pipeline.get_day_of_week_portuguese(date), "Segunda-feira");
        
        let date = NaiveDate::from_ymd_opt(2024, 1, 20).unwrap(); // Saturday
        assert_eq!(pipeline.get_day_of_week_portuguese(date), "Sábado");
    }
    
    #[test]
    fn test_month_name_portuguese() {
        let config = PdwConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = DatabaseManager::new(&db_path).unwrap();
        
        let pipeline = EtlPipeline { config, database };
        
        assert_eq!(pipeline.get_month_name_portuguese(1), "01-Janeiro");
        assert_eq!(pipeline.get_month_name_portuguese(12), "12-Dezembro");
        assert_eq!(pipeline.get_month_name_portuguese(13), "00-Inválido");
    }
    
    #[test]
    fn test_transaction_processing() {
        let config = PdwConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = DatabaseManager::new(&db_path).unwrap();
        
        let pipeline = EtlPipeline { config, database };
        
        let transaction = Transaction {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            transaction_type: Some("ALM".to_string()),
            description: Some("Test; transaction, with∴special chars".to_string()),
            credit: Some(100.555),
            debit: Some(50.999),
            origin: "TestSheet".to_string(),
        };
        
        let processed = pipeline.process_single_transaction(transaction).unwrap().unwrap();
        
        assert_eq!(processed.transaction_type, "ALM");
        assert_eq!(processed.credit, 100.56); // Rounded
        assert_eq!(processed.debit, 51.0); // Rounded
        assert_eq!(processed.description, "Test| transaction| with .'. special chars");
        assert_eq!(processed.day_of_week, "Segunda-feira");
        assert_eq!(processed.month_name, "01-Janeiro");
    }
}