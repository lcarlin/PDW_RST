/*!
# Configuration Management Module

Handles TOML configuration files with backward compatibility for INI format.
Provides validation and migration utilities.
*/

use crate::error::{ConfigError, PdwError};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdwConfig {
    pub directories: DirectoryConfig,
    pub file_types: FileTypeConfig,
    pub settings: SettingsConfig,
}

/// Directory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub dir_in: PathBuf,
    pub dir_out: PathBuf,
    pub database_dir: PathBuf,
    pub log_dir: PathBuf,
}

/// File type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeConfig {
    pub type_in: String,
    pub type_out: String,
    pub db_file_type: String,
    pub log_file: String,
    pub input_file: String,
    pub out_db_file: String,
    pub out_rpt_file: String,
    pub transient_data_file: Option<String>,
}

/// Settings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub current_version: String,
    pub api_version: Option<String>,
    pub guiding_table: String,
    pub types_of_entries: String,
    pub general_entries_table: String,
    pub run_data_loader: bool,
    pub run_reports: bool,
    pub overwrite_db: bool,
    pub create_pivot: bool,
    pub rpt_single_file: bool,
    pub parallels: Option<u32>,
    pub multithreading: bool,
    pub save_discarted_data: bool,
    pub discarted_data_table: String,
    pub anual_pivot_table: String,
    pub full_pivot_table: String,
    pub run_dinamic_report: bool,
    pub din_report_guiding: String,
    pub export_transient_data: bool,
    pub transient_data_table: Option<String>,
    pub transient_data_column: String,
    pub export_other_types: bool,
    pub dayly_progress: String,
    pub splt_paymnt_tab: String,
    pub out_res_pmnt_tab: String,
    pub monthly_summaties: String,
    pub yaml_sql_file: String,
}

impl Default for PdwConfig {
    fn default() -> Self {
        Self {
            directories: DirectoryConfig {
                dir_in: PathBuf::from("./input/"),
                dir_out: PathBuf::from("./output/"),
                database_dir: PathBuf::from("./database/"),
                log_dir: PathBuf::from("./logs/"),
            },
            file_types: FileTypeConfig {
                type_in: "xlsx".to_string(),
                type_out: "xlsx".to_string(),
                db_file_type: "db".to_string(),
                log_file: "PDW.SysMap.log".to_string(),
                input_file: "PDW".to_string(),
                out_db_file: "PDW".to_string(),
                out_rpt_file: "PDW_REPORTS.v2".to_string(),
                transient_data_file: Some("Lancamentos_Gerais_TMP".to_string()),
            },
            settings: SettingsConfig {
                current_version: "9.11.0".to_string(),
                api_version: Some("2.0.0".to_string()),
                guiding_table: "GUIDING".to_string(),
                types_of_entries: "TiposLancamentos".to_string(),
                general_entries_table: "LANCAMENTOS_GERAIS".to_string(),
                run_data_loader: true,
                run_reports: true,
                overwrite_db: true,
                create_pivot: true,
                rpt_single_file: true,
                parallels: Some(89),
                multithreading: false,
                save_discarted_data: false,
                discarted_data_table: "discarted_data".to_string(),
                anual_pivot_table: "HistoricoAnual".to_string(),
                full_pivot_table: "HistoricoGeral".to_string(),
                run_dinamic_report: true,
                din_report_guiding: "General_din_reports".to_string(),
                export_transient_data: false,
                transient_data_table: Some("Transient_data".to_string()),
                transient_data_column: "Origem".to_string(),
                export_other_types: false,
                dayly_progress: "contagem_diaria".to_string(),
                splt_paymnt_tab: "PARCELAMENTOS".to_string(),
                out_res_pmnt_tab: "Resumo_Parcelamentos".to_string(),
                monthly_summaties: "Resumido_In_Out".to_string(),
                yaml_sql_file: "PDW_QUERIES.yaml".to_string(),
            },
        }
    }
}

impl PdwConfig {
    /// Load configuration from TOML file
    pub fn load(path: &Path) -> Result<Self, PdwError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            }.into());
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidFormat {
                message: format!("Failed to read file: {}", e),
            })?;
        
        // Try TOML first
        if let Ok(config) = toml::from_str::<PdwConfig>(&content) {
            return Ok(config);
        }
        
        // If TOML fails, try INI format for backward compatibility
        Self::load_from_ini(path)
    }
    
    /// Load configuration from INI file (backward compatibility)
    pub fn load_from_ini(path: &Path) -> Result<Self, PdwError> {
        let ini = ini::Ini::load_from_file(path)
            .map_err(|e| ConfigError::IniParse(e))?;
        
        let mut config = PdwConfig::default();
        
        // Parse DIRECTORIES section
        if let Some(section) = ini.section(Some("DIRECTORIES")) {
            if let Some(dir_in) = section.get("DIR_IN") {
                config.directories.dir_in = PathBuf::from(dir_in);
            }
            if let Some(dir_out) = section.get("DIR_OUT") {
                config.directories.dir_out = PathBuf::from(dir_out);
            }
            if let Some(database_dir) = section.get("DATABASE_DIR") {
                config.directories.database_dir = PathBuf::from(database_dir);
            }
            if let Some(log_dir) = section.get("LOG_DIR") {
                config.directories.log_dir = PathBuf::from(log_dir);
            }
        }
        
        // Parse FILE_TYPES section
        if let Some(section) = ini.section(Some("FILE_TYPES")) {
            if let Some(type_in) = section.get("TYPE_IN") {
                config.file_types.type_in = type_in.to_string();
            }
            if let Some(type_out) = section.get("TYPE_OUT") {
                config.file_types.type_out = type_out.to_string();
            }
            if let Some(db_file_type) = section.get("DB_FILE_TYPE") {
                config.file_types.db_file_type = db_file_type.to_string();
            }
            if let Some(log_file) = section.get("LOG_FILE") {
                config.file_types.log_file = log_file.to_string();
            }
            if let Some(input_file) = section.get("INPUT_FILE") {
                config.file_types.input_file = input_file.to_string();
            }
            if let Some(out_db_file) = section.get("OUT_DB_FILE") {
                config.file_types.out_db_file = out_db_file.to_string();
            }
            if let Some(out_rpt_file) = section.get("OUT_RPT_FILE") {
                config.file_types.out_rpt_file = out_rpt_file.to_string();
            }
        }
        
        // Parse SETTINGS section
        if let Some(section) = ini.section(Some("SETTINGS")) {
            if let Some(version) = section.get("CURRENT_VERSION") {
                config.settings.current_version = version.to_string();
            }
            if let Some(guiding_table) = section.get("GUIDING_TABLE") {
                config.settings.guiding_table = guiding_table.to_string();
            }
            if let Some(types_of_entries) = section.get("TYPES_OF_ENTRIES") {
                config.settings.types_of_entries = types_of_entries.to_string();
            }
            if let Some(general_entries_table) = section.get("GENERAL_ENTRIES_TABLE") {
                config.settings.general_entries_table = general_entries_table.to_string();
            }
            
            // Parse boolean settings
            config.settings.run_data_loader = section.get("RUN_DATA_LOADER")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true);
            config.settings.run_reports = section.get("RUN_REPORTS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true);
            config.settings.overwrite_db = section.get("OVERWRITE_DB")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true);
            config.settings.create_pivot = section.get("CREATE_PIVOT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(true);
            config.settings.multithreading = section.get("MULTITHREADING")
                .and_then(|s| s.parse().ok())
                .unwrap_or(false);
            
            // Parse other string settings
            if let Some(yaml_file) = section.get("YAML_SQL_FILE") {
                config.settings.yaml_sql_file = yaml_file.to_string();
            }
        }
        
        Ok(config)
    }
    
    /// Save configuration to TOML file
    pub fn save(&self, path: &Path) -> Result<(), PdwError> {
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::InvalidFormat {
                message: format!("Failed to serialize TOML: {}", e),
            })?;
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, toml_content)?;
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), PdwError> {
        // Check version compatibility
        if self.settings.current_version != "9.11.0" {
            return Err(ConfigError::VersionMismatch {
                expected: "9.11.0".to_string(),
                found: self.settings.current_version.clone(),
            }.into());
        }
        
        // Validate directories exist or can be created
        self.validate_directory(&self.directories.dir_in, "DIR_IN")?;
        self.validate_directory(&self.directories.dir_out, "DIR_OUT")?;
        self.validate_directory(&self.directories.database_dir, "DATABASE_DIR")?;
        self.validate_directory(&self.directories.log_dir, "LOG_DIR")?;
        
        // Validate input file exists
        let input_file = self.get_input_file_path();
        if !input_file.exists() {
            return Err(ConfigError::InvalidPath {
                path: input_file.to_string_lossy().to_string(),
                reason: "Input Excel file does not exist".to_string(),
            }.into());
        }
        
        Ok(())
    }
    
    /// Validate a directory path
    fn validate_directory(&self, path: &Path, name: &str) -> Result<(), PdwError> {
        if !path.exists() {
            // Try to create the directory
            if let Err(e) = fs::create_dir_all(path) {
                return Err(ConfigError::InvalidPath {
                    path: path.to_string_lossy().to_string(),
                    reason: format!("Cannot create directory {}: {}", name, e),
                }.into());
            }
        }
        
        // Check if it's actually a directory
        if !path.is_dir() {
            return Err(ConfigError::InvalidPath {
                path: path.to_string_lossy().to_string(),
                reason: format!("{} is not a directory", name),
            }.into());
        }
        
        Ok(())
    }
    
    /// Get full input file path
    pub fn get_input_file_path(&self) -> PathBuf {
        self.directories.dir_in.join(format!(
            "{}.{}",
            self.file_types.input_file,
            self.file_types.type_in
        ))
    }
    
    /// Get full database file path
    pub fn get_database_path(&self) -> PathBuf {
        let filename = if self.settings.overwrite_db {
            format!("{}.{}", self.file_types.out_db_file, self.file_types.db_file_type)
        } else {
            let timestamp = chrono::Local::now().format("%Y%m%d.%H%M%S");
            format!("{}.{}.{}", self.file_types.out_db_file, timestamp, self.file_types.db_file_type)
        };
        
        self.directories.database_dir.join(filename)
    }
    
    /// Get full log file path
    pub fn get_log_file_path(&self) -> PathBuf {
        self.directories.log_dir.join(&self.file_types.log_file)
    }
    
    /// Get YAML queries file path
    pub fn get_yaml_queries_path(&self) -> PathBuf {
        self.directories.dir_in.join(&self.settings.yaml_sql_file)
    }
    
    /// Create a sample TOML configuration file
    pub fn create_sample_config(path: &Path) -> Result<(), PdwError> {
        let config = PdwConfig::default();
        config.save(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_default_config() {
        let config = PdwConfig::default();
        assert_eq!(config.settings.current_version, "9.11.0");
        assert_eq!(config.file_types.type_in, "xlsx");
        assert!(config.settings.run_data_loader);
    }
    
    #[test]
    fn test_toml_serialization() {
        let config = PdwConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: PdwConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.settings.current_version, parsed.settings.current_version);
    }
    
    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let original_config = PdwConfig::default();
        original_config.save(&config_path).unwrap();
        
        let loaded_config = PdwConfig::load(&config_path).unwrap();
        assert_eq!(original_config.settings.current_version, loaded_config.settings.current_version);
    }
    
    #[test]
    fn test_ini_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let ini_path = temp_dir.path().join("test.cfg");
        
        let ini_content = r#"
[DIRECTORIES]
DIR_IN = ./input/
DIR_OUT = ./output/

[FILE_TYPES]
TYPE_IN = xlsx
INPUT_FILE = PDW

[SETTINGS]
CURRENT_VERSION = 9.11.0
RUN_DATA_LOADER = True
"#;
        
        fs::write(&ini_path, ini_content).unwrap();
        let config = PdwConfig::load_from_ini(&ini_path).unwrap();
        assert_eq!(config.settings.current_version, "9.11.0");
        assert!(config.settings.run_data_loader);
    }
    
    #[test]
    fn test_path_generation() {
        let config = PdwConfig::default();
        let input_path = config.get_input_file_path();
        assert!(input_path.to_string_lossy().contains("PDW.xlsx"));
        
        let db_path = config.get_database_path();
        assert!(db_path.to_string_lossy().contains(".db"));
    }
}