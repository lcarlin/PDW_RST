# PDW Rust Build Script
# Checks for Rust installation and builds the project

Write-Host "PDW Rust Build Script" -ForegroundColor Green
Write-Host "=====================" -ForegroundColor Green

# Check if Rust is installed
try {
    $rustVersion = rustc --version 2>$null
    if ($rustVersion) {
        Write-Host "✓ Rust found: $rustVersion" -ForegroundColor Green
    } else {
        throw "Rust not found"
    }
} catch {
    Write-Host "✗ Rust not installed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "After installation, restart your terminal and run this script again." -ForegroundColor Yellow
    exit 1
}

# Check if Cargo is available
try {
    $cargoVersion = cargo --version 2>$null
    if ($cargoVersion) {
        Write-Host "✓ Cargo found: $cargoVersion" -ForegroundColor Green
    } else {
        throw "Cargo not found"
    }
} catch {
    Write-Host "✗ Cargo not available" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Building PDW Rust..." -ForegroundColor Cyan

# Create necessary directories
$directories = @("input", "output", "database", "logs")
foreach ($dir in $directories) {
    if (!(Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
        Write-Host "✓ Created directory: $dir" -ForegroundColor Green
    }
}

# Check project structure
Write-Host ""
Write-Host "Checking project structure..." -ForegroundColor Cyan
cargo check

if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Project structure is valid" -ForegroundColor Green
    
    Write-Host ""
    Write-Host "Building release version..." -ForegroundColor Cyan
    cargo build --release
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "✓ Build completed successfully!" -ForegroundColor Green
        Write-Host "Executable location: target/release/pdw.exe" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "To run PDW:" -ForegroundColor Cyan
        Write-Host "  ./target/release/pdw.exe --help" -ForegroundColor White
        Write-Host "  ./target/release/pdw.exe --config pdw_config.toml" -ForegroundColor White
    } else {
        Write-Host "✗ Build failed" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "✗ Project check failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Common issues:" -ForegroundColor Yellow
    Write-Host "- Missing dependencies (run: cargo fetch)" -ForegroundColor White
    Write-Host "- Syntax errors in source code" -ForegroundColor White
    Write-Host "- Incompatible Rust version (requires 1.70+)" -ForegroundColor White
    exit 1
}