/*!
# Personal Data Warehouse (PDW) - Rust Implementation
 
A high-performance ETL system for processing Excel financial data into SQLite databases
with comprehensive reporting capabilities.

## Version History
- 9.11.0 - Rust implementation with full Python feature parity
- Based on Python PDW version 9.11.0 by Carlin, Luiz A.

## Features
- Excel workbook processing with multiple sheet support
- SQLite database generation with pivot tables
- Multi-format report generation (Excel, CSV, JSON, XML)
- YAML-configurable dynamic reports
- Cross-platform single binary deployment
- Memory-safe processing with Rust's ownership model
*/

use anyhow::Result;
use clap::Parser;
use log::{info, error};
use std::path::PathBuf;
use std::time::Instant;

mod config;
mod database;
mod error;
mod etl;
mod excel;
mod logging;
mod reporting;

use crate::config::PdwConfig;
use crate::etl::EtlPipeline;
use crate::error::PdwError;

/// Personal Data Warehouse - ETL system for Excel to SQLite processing
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path (TOML format)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Dry run - validate configuration without processing
    #[arg(short, long)]
    dry_run: bool,
    
    /// Skip data loading phase
    #[arg(long)]
    skip_loader: bool,
    
    /// Skip report generation phase
    #[arg(long)]
    skip_reports: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    logging::init_logger(args.verbose)?;
    
    let start_time = Instant::now();
    info!("Personal Data Warehouse (Rust) v{} starting", env!("CARGO_PKG_VERSION"));
    
    // Load configuration
    let config_path = args.config.unwrap_or_else(|| PathBuf::from("pdw_config.toml"));
    let config = match PdwConfig::load(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e.into());
        }
    };
    
    info!("Configuration loaded from: {}", config_path.display());
    
    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Configuration validation failed: {}", e);
        return Err(e.into());
    }
    
    if args.dry_run {
        info!("Dry run completed successfully - configuration is valid");
        return Ok(());
    }
    
    // Create ETL pipeline
    let mut pipeline = EtlPipeline::new(config)?;
    
    // Execute ETL phases based on configuration and arguments
    let run_loader = pipeline.config().settings.run_data_loader && !args.skip_loader;
    let run_reports = pipeline.config().settings.run_reports && !args.skip_reports;
    
    if run_loader {
        info!("Starting data loading phase...");
        pipeline.execute_data_loading()?;
        info!("Data loading completed successfully");
    }
    
    if pipeline.config().settings.create_pivot {
        info!("Creating pivot tables...");
        pipeline.create_pivot_tables()?;
        info!("Pivot tables created successfully");
    }
    
    if run_reports {
        info!("Starting report generation...");
        pipeline.generate_reports()?;
        info!("Report generation completed successfully");
    }
    
    let duration = start_time.elapsed();
    info!(
        "PDW processing completed successfully in {:.2} seconds", 
        duration.as_secs_f64()
    );
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_main_with_invalid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.toml");
        fs::write(&config_path, "invalid toml content").unwrap();
        
        let result = PdwConfig::load(&config_path);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_version_info() {
        assert_eq!(env!("CARGO_PKG_VERSION"), "9.11.0");
        assert_eq!(env!("CARGO_PKG_NAME"), "pdw-rust");
    }
}