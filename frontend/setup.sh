#!/bin/bash

# Trading Dashboard Quick Setup Script
# This script sets up and runs the trading dashboard

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Trading Dashboard Setup${NC}"
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${YELLOW}⚠ Node.js not found. Please install Node.js 16+${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Node.js found: $(node --version)${NC}"
echo ""

# Navigate to frontend directory
cd "$(dirname "$0")"

# Install dependencies
echo -e "${BLUE}📦 Installing dependencies...${NC}"
npm install

echo -e "${BLUE}🔧 Setup complete!${NC}"
echo ""
echo -e "${GREEN}✓ To start the development server, run:${NC}"
echo "  ${BLUE}npm run dev${NC}"
echo ""
echo -e "${GREEN}✓ To build for production, run:${NC}"
echo "  ${BLUE}npm run build${NC}"
echo ""
echo -e "${YELLOW}⚠  Make sure your Rust backend is running on port 3000:${NC}"
echo "  ${BLUE}cd ../backend && cargo run --release${NC}"
echo ""
echo -e "${GREEN}📈 Dashboard will be available at: http://localhost:5173${NC}"
