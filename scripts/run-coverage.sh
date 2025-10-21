#!/bin/bash
#
# Run code coverage report locally
#
# Best practices 2025:
# - Use tarpaulin for accurate coverage
# - Generate HTML and XML reports
# - Exclude test files from coverage
# - Set reasonable timeout for large test suites

set -e

echo "🔍 Running code coverage analysis..."

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Clean previous coverage data
rm -rf coverage/
mkdir -p coverage/

# Run coverage
echo "🧪 Generating coverage report..."
cargo tarpaulin \
    --out Html \
    --out Xml \
    --output-dir coverage \
    --all-features \
    --workspace \
    --timeout 300 \
    --exclude-files 'tests/*' \
    --exclude-files 'benches/*' \
    --exclude-files 'examples/*' \
    -- --test-threads 1

# Generate coverage summary
echo ""
echo "📊 Coverage Summary:"
echo "===================="

# Extract coverage percentage from XML
if [ -f coverage/cobertura.xml ]; then
    # Parse coverage percentage (basic parsing)
    grep -o 'line-rate="[0-9.]*"' coverage/cobertura.xml | head -1 | \
        sed 's/line-rate="//;s/"//' | \
        awk '{printf "Line Coverage: %.2f%%\n", $1 * 100}'
fi

echo ""
echo "✅ Coverage report generated!"
echo "📁 HTML Report: coverage/index.html"
echo "📁 XML Report: coverage/cobertura.xml"
echo ""
echo "💡 To view HTML report:"
echo "   open coverage/index.html     # macOS"
echo "   xdg-open coverage/index.html # Linux"
echo ""

# Open report if on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🌐 Opening coverage report..."
    open coverage/index.html
fi
