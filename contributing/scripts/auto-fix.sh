#!/bin/bash
# Auto-fix common issues before committing
# This modifies files to fix formatting and linting issues

set -e

# Match GitHub Actions environment
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1

echo "🔧 Auto-fixing common issues..."
echo "================================"
echo ""

# Auto-format code
echo "1️⃣ Auto-formatting code..."
cargo fmt
echo "✓ Code formatted"

echo ""
echo "2️⃣ Auto-fixing clippy issues..."
cargo clippy --fix --allow-dirty --allow-staged
cargo clippy --fix --allow-dirty --allow-staged --tests
echo "✓ Clippy fixes applied (where possible)"

echo ""
echo "3️⃣ Checking if all issues are fixed..."
echo ""

# Run quick check to verify
echo "Running quick-check to verify fixes..."
echo "--------------------------------------"
./contributing/scripts/quick-check.sh

echo ""
echo "🎉 Auto-fix complete!"
echo ""
echo "💡 Next steps:"
echo "   - Review the changes with 'git diff'"
echo "   - Stage changes with 'git add -p' (interactive) or 'git add .'"
echo "   - Commit with a descriptive message"