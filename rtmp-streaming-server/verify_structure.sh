#!/bin/bash

echo "Verifying RTMP Streaming Server structure..."
echo "==========================================="

# Check if all required files exist
echo "1. Checking file structure:"

required_files=(
    "Cargo.toml"
    "README.md"
    "src/main.rs"
    "src/lib.rs"
    "src/error.rs"
    "src/server.rs"
    "src/session.rs"
    "src/stream.rs"
    "src/protocol.rs"
)

all_good=true
for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✓ $file"
    else
        echo "  ✗ $file - MISSING"
        all_good=false
    fi
done

echo ""
echo "2. Checking file contents:"

# Check Cargo.toml for required dependencies
echo "  Checking Cargo.toml dependencies..."
if grep -q "rtmp" Cargo.toml && \
   grep -q "tokio" Cargo.toml && \
   grep -q "clap" Cargo.toml; then
    echo "    ✓ Core dependencies defined"
else
    echo "    ✗ Missing core dependencies"
    all_good=false
fi

# Check main.rs for basic structure
echo "  Checking main.rs structure..."
if grep -q "fn main()" src/main.rs && \
   grep -q "clap::Parser" src/main.rs && \
   grep -q "tracing" src/main.rs; then
    echo "    ✓ Main entry point structured"
else
    echo "    ✗ Main entry point incomplete"
    all_good=false
fi

echo ""
echo "3. Summary:"
if [ "$all_good" = true ]; then
    echo "  ✓ All files present and structured correctly"
    echo "  ✓ RTMP server framework implemented"
    echo "  ✓ Next step: RTMP protocol implementation"
else
    echo "  ✗ Some issues found in structure"
fi

echo ""
echo "Project location: /workspace/rtmp-streaming-server"
echo "Next phase: Implement RTMP protocol handlers"