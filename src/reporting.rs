/*!
# Reporting Module

Handles report generation in multiple formats (Excel, CSV, JSON, XML)
using YAML-defined queries and templates.
*/

use crate::config::PdwConfig;
use crate::database::DatabaseManager;
use crate::error::{ReportError, PdwError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Report generator
pub struct ReportGenerator {
    database: DatabaseManager,
    config: PdwConfig,
}

/// YAML query configuration
#[derive(Debug, Deserialize, Serialize)]
pub struct QueryConfig {
    #[serde(default)]
    pub queries_gera_hist: Vec<QueryDefinition>,
    #[serde(default)]
    pub queries_padrao: Vec<QueryDefinition>,
}

/// Individual query definition
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QueryDefinition {
    pub sql: String,
    pub sheet_name: String,
}

impl ReportGenerator {
    /// Create new report generator
    pub fn new(database: DatabaseManager, config: PdwConfig) -> Self {
        Self { database, config }
    }
    
    /// Load queries from YAML file
    pub fn load_queries(&self) -> Result<QueryConfig, PdwError> {
        let yaml_path = self.config.get_yaml_queries_path();
        
        if !yaml_path.exists() {
            return Err(ReportError::YamlQueryFile {
                path: yaml_path.to_string_lossy().to_string(),
                reason: "File not found".to_string(),
            }.into());
        }
        
        let content = std::fs::read_to_string(&yaml_path)
            .map_err(|e| ReportError::YamlQueryFile {
                path: yaml_path.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        
        let config: QueryConfig = serde_yaml::from_str(&content)
            .map_err(|e| ReportError::YamlParse(e))?;
        
        Ok(config)
    }
    
    /// Generate Excel reports
    pub fn generate_excel_reports(&self) -> Result<(), PdwError> {
        let query_config = self.load_queries()?;
        let output_path = self.config.directories.dir_out.join(format!(
            "{}.{}",
            self.config.file_types.out_rpt_file,
            self.config.file_types.type_out
        ));
        
        // Create Excel workbook
        let mut workbook = rust_xlsxwriter::Workbook::new();
        
        // Variable substitution map
        let variables = self.create_variable_map();
        
        // Process conditional queries (gera_hist)
        if self.config.settings.create_pivot {
            for query_def in &query_config.queries_gera_hist {
                let sql = self.substitute_variables(&query_def.sql, &variables);
                let sheet_name = self.substitute_variables(&query_def.sheet_name, &variables);
                
                self.add_query_to_workbook(&mut workbook, &sql, &sheet_name)?;
            }
        }
        
        // Process standard queries
        for query_def in &query_config.queries_padrao {
            let sql = self.substitute_variables(&query_def.sql, &variables);
            let sheet_name = &query_def.sheet_name;
            
            self.add_query_to_workbook(&mut workbook, &sql, sheet_name)?;
        }
        
        // Process dynamic reports if enabled
        if self.config.settings.run_dinamic_report {
            self.add_dynamic_reports_to_workbook(&mut workbook)?;
        }
        
        // Save workbook
        workbook.save(&output_path)
            .map_err(|e| ReportError::ExcelWriter(e))?;
        
        log::info!("Excel reports generated: {}", output_path.display());
        Ok(())
    }
    
    /// Add query results to Excel workbook
    fn add_query_to_workbook(
        &self,
        workbook: &mut rust_xlsxwriter::Workbook,
        sql: &str,
        sheet_name: &str,
    ) -> Result<(), PdwError> {
        let results = self.database.execute_query(sql)?;
        
        if results.is_empty() {
            return Ok(());
        }
        
        let mut worksheet = workbook.add_worksheet();
        worksheet.set_name(sheet_name)
            .map_err(|e| ReportError::ExcelWriter(e))?;
        
        // Write data to worksheet
        for (row_idx, row_data) in results.iter().enumerate() {
            for (col_idx, cell_value) in row_data.iter().enumerate() {
                let value = match cell_value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => cell_value.to_string(),
                };
                
                worksheet.write_string(row_idx as u32, col_idx as u16, &value)
                    .map_err(|e| ReportError::ExcelWriter(e))?;
            }
        }
        
        Ok(())
    }
    
    /// Add dynamic reports to workbook
    fn add_dynamic_reports_to_workbook(
        &self,
        workbook: &mut rust_xlsxwriter::Workbook,
    ) -> Result<(), PdwError> {
        let dynamic_reports_query = format!(
            "SELECT * FROM {}",
            self.config.settings.din_report_guiding
        );
        
        let dynamic_reports = self.database.execute_query(&dynamic_reports_query)?;
        
        for report_row in dynamic_reports {
            if report_row.len() >= 2 {
                if let (Some(Value::String(dest_table)), Some(Value::String(report_name))) = 
                    (report_row.get(0), report_row.get(1)) {
                    
                    let query = format!("SELECT * FROM {}", dest_table);
                    self.add_query_to_workbook(workbook, &query, report_name)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Export data to CSV format
    pub fn export_csv(&self, query: &str, output_path: &Path) -> Result<(), PdwError> {
        let results = self.database.execute_query(query)?;
        
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .from_path(output_path)
            .map_err(|e| ReportError::CsvWriter(e))?;
        
        for row_data in results {
            let string_row: Vec<String> = row_data.iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string().replace(".", ","), // Portuguese decimal format
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => v.to_string(),
                })
                .collect();
            
            writer.write_record(&string_row)
                .map_err(|e| ReportError::CsvWriter(e))?;
        }
        
        writer.flush()
            .map_err(|e| ReportError::CsvWriter(e))?;
        
        Ok(())
    }
    
    /// Export data to JSON format
    pub fn export_json(&self, query: &str, output_path: &Path) -> Result<(), PdwError> {
        let results = self.database.execute_query(query)?;
        
        let json_data = serde_json::to_string_pretty(&results)
            .map_err(|e| ReportError::JsonSerialization(e))?;
        
        std::fs::write(output_path, json_data)?;
        
        // Compress if configured
        if self.config.settings.export_other_types {
            self.compress_file(output_path)?;
        }
        
        Ok(())
    }
    
    /// Export data to XML format
    pub fn export_xml(&self, query: &str, output_path: &Path) -> Result<(), PdwError> {
        let results = self.database.execute_query(query)?;
        
        let mut xml_content = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<data>\n");
        
        for row_data in results {
            xml_content.push_str("   <item>\n");
            
            for (idx, cell_value) in row_data.iter().enumerate() {
                let value = match cell_value {
                    Value::String(s) => xml_escape(s),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => xml_escape(&cell_value.to_string()),
                };
                
                xml_content.push_str(&format!("      <col{}>{}</col{}>\n", idx + 1, value, idx + 1));
            }
            
            xml_content.push_str("   </item>\n");
        }
        
        xml_content.push_str("</data>\n");
        
        std::fs::write(output_path, xml_content)?;
        
        // Compress if configured
        if self.config.settings.export_other_types {
            self.compress_file(output_path)?;
        }
        
        Ok(())
    }
    
    /// Export general entries to multiple formats
    pub fn export_general_entries(&self) -> Result<(), PdwError> {
        let base_filename = format!("{}.v2", self.config.settings.general_entries_table);
        let base_path = self.config.directories.dir_out.join(&base_filename);
        
        let query = format!(
            "SELECT 
                substr(LG.Data, 9, 2) || '-' || substr(LG.Data, 6, 2) || '-' || substr(LG.Data, 1, 4) AS Quando,
                LG.DIA_SEMANA as 'Dia da Semana',
                LG.TIPO as 'Tipo',
                LG.DESCRICAO as 'Descricao/Lancamento',
                replace(LG.Credito, '.', ',') as 'Credito',
                replace(LG.Debito, '.', ',') as 'Debito',
                char(39) || cast(Mes as text) as 'Mes',
                char(39) || cast(Ano as text) as 'Ano',
                char(39) || MES_EXTENSO as 'Mes(Por Extenso)',
                char(39) || cast(AnoMes as text) as 'Ano/Mes',
                LG.Origem as Origem
            FROM {} LG 
            ORDER BY Data DESC",
            self.config.settings.general_entries_table
        );
        
        // Export CSV
        let csv_path = base_path.with_extension("csv");
        self.export_csv(&query, &csv_path)?;
        
        // Export other formats if enabled
        if self.config.settings.export_other_types {
            let json_path = base_path.with_extension("json");
            self.export_json(&query, &json_path)?;
            
            let xml_path = base_path.with_extension("xml");
            self.export_xml(&query, &xml_path)?;
        }
        
        Ok(())
    }
    
    /// Create variable substitution map
    fn create_variable_map(&self) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        
        variables.insert("entries_table".to_string(), self.config.settings.general_entries_table.clone());
        variables.insert("full_hist".to_string(), self.config.settings.full_pivot_table.clone());
        variables.insert("anual_hist".to_string(), self.config.settings.anual_pivot_table.clone());
        variables.insert("day_prog".to_string(), self.config.settings.dayly_progress.clone());
        variables.insert("splt_pmnt_res".to_string(), self.config.settings.out_res_pmnt_tab.clone());
        variables.insert("mont_summ".to_string(), self.config.settings.monthly_summaties.clone());
        variables.insert("dyn_rep_tab".to_string(), self.config.settings.din_report_guiding.clone());
        
        variables
    }
    
    /// Substitute variables in SQL query
    fn substitute_variables(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
    
    /// Compress file using gzip
    fn compress_file(&self, file_path: &Path) -> Result<(), PdwError> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let input_data = std::fs::read(file_path)?;
        let compressed_path = file_path.with_extension(
            format!("{}.gz", file_path.extension().unwrap_or_default().to_string_lossy())
        );
        
        let output_file = File::create(&compressed_path)?;
        let mut encoder = GzEncoder::new(output_file, Compression::default());
        encoder.write_all(&input_data)?;
        encoder.finish()?;
        
        // Remove original file
        std::fs::remove_file(file_path)?;
        
        log::info!("Compressed file created: {}", compressed_path.display());
        Ok(())
    }
}

/// Trait for report operations
pub trait ReportOperations {
    fn load_queries(&self) -> Result<QueryConfig, PdwError>;
    fn generate_excel_reports(&self) -> Result<(), PdwError>;
    fn export_csv(&self, query: &str, output_path: &Path) -> Result<(), PdwError>;
    fn export_json(&self, query: &str, output_path: &Path) -> Result<(), PdwError>;
}

impl ReportOperations for ReportGenerator {
    fn load_queries(&self) -> Result<QueryConfig, PdwError> {
        self.load_queries()
    }
    
    fn generate_excel_reports(&self) -> Result<(), PdwError> {
        self.generate_excel_reports()
    }
    
    fn export_csv(&self, query: &str, output_path: &Path) -> Result<(), PdwError> {
        self.export_csv(query, output_path)
    }
    
    fn export_json(&self, query: &str, output_path: &Path) -> Result<(), PdwError> {
        self.export_json(query, output_path)
    }
}

/// Escape XML special characters
fn xml_escape(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("test & <data>"), "test &amp; &lt;data&gt;");
        assert_eq!(xml_escape("'quoted'"), "&apos;quoted&apos;");
    }
    
    #[test]
    fn test_variable_substitution() {
        let config = PdwConfig::default();
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database = DatabaseManager::new(&db_path).unwrap();
        
        let generator = ReportGenerator::new(database, config);
        let variables = generator.create_variable_map();
        
        let template = "SELECT * FROM {entries_table} WHERE date > '{full_hist}'";
        let result = generator.substitute_variables(template, &variables);
        
        assert!(result.contains("LANCAMENTOS_GERAIS"));
        assert!(result.contains("HistoricoGeral"));
    }
    
    #[test]
    fn test_query_config_deserialization() {
        let yaml_content = r#"
queries_padrao:
  - sql: "SELECT * FROM test"
    sheet_name: "TestSheet"
queries_gera_hist:
  - sql: "SELECT * FROM {entries_table}"
    sheet_name: "HistorySheet"
"#;
        
        let config: QueryConfig = serde_yaml::from_str(yaml_content).unwrap();
        assert_eq!(config.queries_padrao.len(), 1);
        assert_eq!(config.queries_gera_hist.len(), 1);
        assert_eq!(config.queries_padrao[0].sheet_name, "TestSheet");
    }
}