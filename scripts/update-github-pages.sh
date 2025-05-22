#!/bin/bash
# Script to help verify and update GitHub Pages settings after repository renaming
# This script will guide you through the manual steps needed to update GitHub Pages

set -e

echo "GitHub Pages Update Helper for repository renaming"
echo "=================================================="
echo 
echo "The following steps will help you reconfigure GitHub Pages after renaming the repository from 'orbitrs' to 'orbit'"
echo

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "The GitHub CLI (gh) is not installed. It's recommended for easier authentication."
    echo "You can install it from https://cli.github.com/"
    echo
    echo "Continuing with manual instructions..."
else
    echo "GitHub CLI detected. You can use it to authenticate and view repository settings."
    echo "Try running: gh auth login"
    echo "And then: gh repo view orbitrs/orbit --web"
    echo
fi

echo "STEPS TO UPDATE GITHUB PAGES:"
echo "============================="
echo
echo "1. Go to the GitHub repository settings:"
echo "   https://github.com/orbitrs/orbit/settings/pages"
echo
echo "2. Under 'Build and deployment':"
echo "   - Source: Select 'GitHub Actions'"
echo "   - This will use our new pages.yml workflow"
echo
echo "3. After saving settings, manually trigger the GitHub Pages workflow:"
echo "   https://github.com/orbitrs/orbit/actions/workflows/pages.yml"
echo "   Click 'Run workflow' button"
echo
echo "4. Wait for the workflow to complete and then check:"
echo "   https://orbitrs.github.io/orbit/"
echo
echo "If you still encounter issues, you might need to:"
echo "- Check branch permissions and workflow permissions in repository settings"
echo "- Verify that GitHub Pages is enabled for your organization"
echo "- Try running with a clean gh-pages branch history"
echo
echo "For completely resetting gh-pages branch (if needed):"
echo "git checkout --orphan gh-pages-new"
echo "git rm -rf ."
echo "echo '# Documentation' > README.md"
echo "git add README.md"
echo "git commit -m 'Initial gh-pages branch'"
echo "git push origin gh-pages-new:gh-pages -f"
echo

read -p "Would you like to trigger the GitHub Pages workflow now? (y/n): " trigger_workflow

if [[ "$trigger_workflow" == "y" ]]; then
    if command -v gh &> /dev/null; then
        echo "Triggering GitHub Pages workflow..."
        gh workflow run pages.yml --repo orbitrs/orbit
        echo "Workflow triggered! Check status at: https://github.com/orbitrs/orbit/actions"
    else
        echo "GitHub CLI not available. Please trigger the workflow manually at:"
        echo "https://github.com/orbitrs/orbit/actions/workflows/pages.yml"
    fi
fi

echo
echo "Done! GitHub Pages should be updated within a few minutes after the workflow completes."
