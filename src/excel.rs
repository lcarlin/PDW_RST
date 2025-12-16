/*!
# Excel Processing Module

Handles Excel file reading and parsing using the calamine crate.
Provides functionality for reading guiding sheets, accounting data, and reference data.
*/

use crate::error::{ExcelError, PdwError};
use calamine::{Reader, Xlsx, open_workbook, DataType, Range};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Excel processor for reading workbooks
pub struct ExcelProcessor {
    workbook: Xlsx<std::io::BufReader<std::fs::File>>,
}

/// Configuration for sheet processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetConfig {
    pub table_name: String,
    pub is_accounting: bool,
    pub is_loadable: bool,
}

/// Financial transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub date: Option<NaiveDate>,
    pub transaction_type: Option<String>,
    pub description: Option<String>,
    pub credit: Option<f64>,
    pub debit: Option<f64>,
    pub origin: String,
}

/// Raw sheet data
#[derive(Debug, Clone)]
pub struct SheetData {
    pub name: String,
    pub data: Vec<Vec<DataType>>,
    pub is_accounting: bool,
    pub is_loadable: bool,
}

impl ExcelProcessor {
    /// Open Excel workbook
    pub fn new(path: &Path) -> Result<Self, PdwError> {
        let workbook = open_workbook(path)
            .map_err(|e| ExcelError::FileOpen {
                path: path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(Self { workbook })
    }
    
    /// Get list of sheet names
    pub fn get_sheet_names(&self) -> Vec<String> {
        self.workbook.sheet_names().to_vec()
    }
    
    /// Read guiding sheet configuration
    pub fn read_guiding_sheet(&mut self, sheet_name: &str) -> Result<Vec<SheetConfig>, PdwError> {
        let range = self.get_sheet_range(sheet_name)?;
        let mut configs = Vec::new();
        
        // Skip header row, start from row 1
        for row_idx in 1..range.height() {
            if let Some(row) = range.rows().nth(row_idx) {
                if row.len() >= 3 {
                    let table_name = self.cell_to_string(&row[0]);
                    let accounting = self.cell_to_string(&row[1]);
                    let loadable = self.cell_to_string(&row[2]);
                    
                    if !table_name.is_empty() {
                        configs.push(SheetConfig {
                            table_name,
                            is_accounting: accounting.trim().to_uppercase() == "X",
                            is_loadable: loadable.trim().to_uppercase() == "X",
                        });
                    }
                }
            }
        }
        
        Ok(configs)
    }
    
    /// Read accounting sheet data
    pub fn read_accounting_sheet(&mut self, sheet_name: &str) -> Result<Vec<Transaction>, PdwError> {
        let range = self.get_sheet_range(sheet_name)?;
        let mut transactions = Vec::new();
        
        // Expected columns: Data, TIPO, DESCRICAO, Credito, Debito
        for row_idx in 1..range.height() {
            if let Some(row) = range.rows().nth(row_idx) {
                if row.len() >= 5 {
                    let date = self.cell_to_date(&row[0]);
                    let transaction_type = self.cell_to_string_option(&row[1]);
                    let description = self.cell_to_string_option(&row[2]);
                    let credit = self.cell_to_float(&row[3]);
                    let debit = self.cell_to_float(&row[4]);
                    
                    // Only add transaction if it has essential data
                    if date.is_some() || transaction_type.is_some() {
                        transactions.push(Transaction {
                            date,
                            transaction_type,
                            description,
                            credit,
                            debit,
                            origin: sheet_name.to_string(),
                        });
                    }
                }
            }
        }
        
        Ok(transactions)
    }
    
    /// Read reference sheet data (non-accounting)
    pub fn read_reference_sheet(&mut self, sheet_name: &str) -> Result<Vec<Vec<String>>, PdwError> {
        let range = self.get_sheet_range(sheet_name)?;
        let mut data = Vec::new();
        
        for row in range.rows() {
            let row_data: Vec<String> = row.iter()
                .map(|cell| self.cell_to_string(cell))
                .collect();
            data.push(row_data);
        }
        
        Ok(data)
    }
    
    /// Get sheet range
    fn get_sheet_range(&mut self, sheet_name: &str) -> Result<Range<DataType>, PdwError> {
        self.workbook
            .worksheet_range(sheet_name)
            .map_err(|e| ExcelError::SheetNotFound {
                sheet_name: sheet_name.to_string(),
            })?
            .map_err(|e| ExcelError::InvalidStructure {
                sheet_name: sheet_name.to_string(),
                reason: e.to_string(),
            })
    }
    
    /// Convert cell to string
    fn cell_to_string(&self, cell: &DataType) -> String {
        match cell {
            DataType::String(s) => s.clone(),
            DataType::Float(f) => f.to_string(),
            DataType::Int(i) => i.to_string(),
            DataType::Bool(b) => b.to_string(),
            DataType::DateTime(dt) => dt.to_string(),
            DataType::Error(_) => String::new(),
            DataType::Empty => String::new(),
        }
    }
    
    /// Convert cell to optional string
    fn cell_to_string_option(&self, cell: &DataType) -> Option<String> {
        let s = self.cell_to_string(cell);
        if s.is_empty() {
            None
        } else {
            Some(s)
        }
    }
    
    /// Convert cell to date
    fn cell_to_date(&self, cell: &DataType) -> Option<NaiveDate> {
        match cell {
            DataType::DateTime(dt) => Some(dt.date()),
            DataType::Float(f) => {
                // Excel date serial number
                let base_date = NaiveDate::from_ymd_opt(1900, 1, 1)?;
                base_date.checked_add_signed(chrono::Duration::days(*f as i64 - 2))
            }
            DataType::String(s) => {
                // Try to parse various date formats
                self.parse_date_string(s)
            }
            _ => None,
        }
    }
    
    /// Convert cell to float
    fn cell_to_float(&self, cell: &DataType) -> Option<f64> {
        match cell {
            DataType::Float(f) => Some(*f),
            DataType::Int(i) => Some(*i as f64),
            DataType::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    /// Parse date from string
    fn parse_date_string(&self, s: &str) -> Option<NaiveDate> {
        // Try common date formats
        let formats = [
            "%Y-%m-%d",
            "%d/%m/%Y",
            "%m/%d/%Y",
            "%d-%m-%Y",
            "%Y/%m/%d",
        ];
        
        for format in &formats {
            if let Ok(date) = NaiveDate::parse_from_str(s, format) {
                return Some(date);
            }
        }
        
        None
    }
}

/// Trait for Excel reading operations
pub trait ExcelReader {
    fn open_workbook(path: &Path) -> Result<Self, PdwError>
    where
        Self: Sized;
    
    fn read_guiding_sheet(&mut self, sheet_name: &str) -> Result<Vec<SheetConfig>, PdwError>;
    fn read_accounting_sheet(&mut self, sheet_name: &str) -> Result<Vec<Transaction>, PdwError>;
    fn read_reference_sheet(&mut self, sheet_name: &str) -> Result<Vec<Vec<String>>, PdwError>;
}

impl ExcelReader for ExcelProcessor {
    fn open_workbook(path: &Path) -> Result<Self, PdwError> {
        Self::new(path)
    }
    
    fn read_guiding_sheet(&mut self, sheet_name: &str) -> Result<Vec<SheetConfig>, PdwError> {
        self.read_guiding_sheet(sheet_name)
    }
    
    fn read_accounting_sheet(&mut self, sheet_name: &str) -> Result<Vec<Transaction>, PdwError> {
        self.read_accounting_sheet(sheet_name)
    }
    
    fn read_reference_sheet(&mut self, sheet_name: &str) -> Result<Vec<Vec<String>>, PdwError> {
        self.read_reference_sheet(sheet_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_cell_conversions() {
        let processor = ExcelProcessor {
            workbook: open_workbook("test.xlsx").unwrap_or_else(|_| {
                // Create a mock workbook for testing
                panic!("Test requires a valid Excel file");
            }),
        };
        
        // Test string conversion
        let cell = DataType::String("test".to_string());
        assert_eq!(processor.cell_to_string(&cell), "test");
        
        // Test float conversion
        let cell = DataType::Float(123.45);
        assert_eq!(processor.cell_to_float(&cell), Some(123.45));
        
        // Test empty cell
        let cell = DataType::Empty;
        assert_eq!(processor.cell_to_string(&cell), "");
    }
    
    #[test]
    fn test_date_parsing() {
        let processor = ExcelProcessor {
            workbook: open_workbook("test.xlsx").unwrap_or_else(|_| {
                panic!("Test requires a valid Excel file");
            }),
        };
        
        // Test date string parsing
        let date = processor.parse_date_string("2024-01-15");
        assert!(date.is_some());
        
        let date = processor.parse_date_string("15/01/2024");
        assert!(date.is_some());
        
        let date = processor.parse_date_string("invalid");
        assert!(date.is_none());
    }
    
    #[test]
    fn test_sheet_config() {
        let config = SheetConfig {
            table_name: "TestSheet".to_string(),
            is_accounting: true,
            is_loadable: true,
        };
        
        assert_eq!(config.table_name, "TestSheet");
        assert!(config.is_accounting);
        assert!(config.is_loadable);
    }
    
    #[test]
    fn test_transaction_creation() {
        let transaction = Transaction {
            date: Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
            transaction_type: Some("ALM".to_string()),
            description: Some("Test transaction".to_string()),
            credit: Some(100.0),
            debit: None,
            origin: "TestSheet".to_string(),
        };
        
        assert!(transaction.date.is_some());
        assert_eq!(transaction.origin, "TestSheet");
    }
}