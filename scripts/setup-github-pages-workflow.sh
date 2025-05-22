#!/bin/bash
# Script to set up GitHub Pages workflow in the orbitrs/.github repository

set -e

WORKSPACE_DIR="/Volumes/EXT/repos/orbitrs"
GITHUB_REPO_DIR="$WORKSPACE_DIR/.github"
WORKSPACE_FILE="$WORKSPACE_DIR/orbitui/orbitrs.code-workspace"

echo "Setting up GitHub Pages workflow for orbitrs organization"
echo "========================================================"
echo

# Step 1: Check if .github repo exists locally, clone or create if needed
if [ -d "$GITHUB_REPO_DIR" ]; then
    echo "Found existing .github directory at $GITHUB_REPO_DIR"
    echo "Updating repository..."
    cd "$GITHUB_REPO_DIR"
    git pull origin main || echo "Failed to pull, repository might be new or disconnected from remote"
else
    echo ".github directory not found, checking if it exists on GitHub..."
    
    # Check if repo exists on GitHub
    if gh repo view orbitrs/.github --json name &>/dev/null; then
        echo "Repository exists on GitHub. Cloning..."
        cd "$WORKSPACE_DIR"
        git clone https://github.com/orbitrs/.github.git
    else
        echo "Repository does not exist on GitHub. Creating new repository..."
        mkdir -p "$GITHUB_REPO_DIR"
        cd "$GITHUB_REPO_DIR"
        git init
        echo "# .github repository for orbitrs organization" > README.md
        git add README.md
        git commit -m "Initial commit"
        
        # Create the repository on GitHub
        echo "Creating GitHub repository orbitrs/.github..."
        gh repo create orbitrs/.github --public --source=. --push --description "Organization-wide GitHub configuration for orbitrs"
    fi
fi

# Step 2: Create workflows directory if it doesn't exist
mkdir -p "$GITHUB_REPO_DIR/workflows"

# Step 3: Create GitHub Pages workflow file
cat > "$GITHUB_REPO_DIR/workflows/pages.yml" << 'EOF'
name: GitHub Pages Deployment

on:
  workflow_dispatch:
  push:
    branches:
      - main
    paths:
      - 'docs/**'
      - '**.md'
      - '.github/workflows/pages.yml'

permissions:
  contents: read
  pages: write
  id-token: write

# Allow one concurrent deployment
concurrency:
  group: "pages"
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v3
        
      - name: Setup Pages
        uses: actions/configure-pages@v3
        
      - name: Setup mdBook
        uses: peaceiris/actions-mdbook@v1
        with:
          mdbook-version: 'latest'
          
      - name: Build with mdBook
        run: |
          cd docs
          mdbook build
          
      - name: Upload artifact
        uses: actions/upload-pages-artifact@v1
        with:
          path: 'docs/book'
          
  deploy:
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    runs-on: ubuntu-latest
    needs: build
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v2
EOF

echo "Created GitHub Pages workflow file at $GITHUB_REPO_DIR/workflows/pages.yml"

# Step 4: Commit and push changes
cd "$GITHUB_REPO_DIR"
git add workflows/pages.yml
git commit -m "Add GitHub Pages workflow"
git push origin HEAD

# Step 5: Update VSCode workspace file to include the .github repo
if [ -f "$WORKSPACE_FILE" ]; then
    echo "Updating VSCode workspace file to include .github repository..."
    # Check if workspace file is a valid JSON
    if jq empty "$WORKSPACE_FILE" 2>/dev/null; then
        # Check if .github folder is already in the workspace
        if ! jq -e '.folders[] | select(.path | contains(".github"))' "$WORKSPACE_FILE" >/dev/null; then
            # Create a temporary file with updated workspace configuration
            jq '.folders += [{"path": "../.github"}]' "$WORKSPACE_FILE" > "$WORKSPACE_FILE.tmp"
            mv "$WORKSPACE_FILE.tmp" "$WORKSPACE_FILE"
            echo "Added .github repository to VSCode workspace"
        else
            echo ".github repository is already in the VSCode workspace"
        fi
    else
        echo "Warning: Workspace file is not valid JSON, cannot update automatically"
        echo "Please manually add the .github repository to your VSCode workspace"
    fi
else
    echo "Warning: VSCode workspace file not found at $WORKSPACE_FILE"
    echo "Please manually add the .github repository to your VSCode workspace"
fi

echo
echo "Complete! GitHub Pages workflow has been set up in orbitrs/.github repository."
echo "To use GitHub Pages for documentation:"
echo "1. Configure GitHub Pages in repository settings to use GitHub Actions"
echo "2. Organize documentation in a 'docs' folder with mdBook structure"
echo "3. Trigger the workflow manually or by pushing changes to documentation files"
echo
