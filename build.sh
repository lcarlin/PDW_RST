#!/bin/bash
# PDW Rust Build Script for Linux/macOS
# Checks for Rust installation and builds the project

echo "PDW Rust Build Script"
echo "====================="

# Check if Rust is installed
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    echo "✓ Rust found: $RUST_VERSION"
else
    echo "✗ Rust not installed"
    echo ""
    echo "Please install Rust from: https://rustup.rs/"
    echo "After installation, restart your terminal and run this script again."
    exit 1
fi

# Check if Cargo is available
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    echo "✓ Cargo found: $CARGO_VERSION"
else
    echo "✗ Cargo not available"
    exit 1
fi

echo ""
echo "Building PDW Rust..."

# Create necessary directories
DIRECTORIES=("input" "output" "database" "logs")
for dir in "${DIRECTORIES[@]}"; do
    if [ ! -d "$dir" ]; then
        mkdir -p "$dir"
        echo "✓ Created directory: $dir"
    fi
done

# Check project structure
echo ""
echo "Checking project structure..."
cargo check

if [ $? -eq 0 ]; then
    echo "✓ Project structure is valid"
    
    echo ""
    echo "Building release version..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        echo ""
        echo "✓ Build completed successfully!"
        echo "Executable location: target/release/pdw"
        echo ""
        echo "To run PDW:"
        echo "  ./target/release/pdw --help"
        echo "  ./target/release/pdw --config pdw_config.toml"
    else
        echo "✗ Build failed"
        exit 1
    fi
else
    echo "✗ Project check failed"
    echo ""
    echo "Common issues:"
    echo "- Missing dependencies (run: cargo fetch)"
    echo "- Syntax errors in source code"
    echo "- Incompatible Rust version (requires 1.70+)"
    exit 1
fi