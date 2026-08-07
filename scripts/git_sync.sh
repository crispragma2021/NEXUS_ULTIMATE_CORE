#!/bin/bash
# NEXUS_SKILL: GitHub Autopilot (Sync)
# Automatically syncs current directory to GitHub

# Configuration
BRANCH=$(git branch --show-current)
if [ -z "$BRANCH" ]; then
    BRANCH="main" # Fallback
fi
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
COMMIT_MSG="chore: auto-backup $TIMESTAMP [Nexus Autopilot]"

# Check if git repository
if [ ! -d .git ]; then
    echo "❌ Error: Not a git repository."
    exit 1
fi

# Check status
if [ -z "$(git status --porcelain)" ]; then
    echo "✅ No changes to sync."
    exit 0
fi

# Add changes
echo "🔄 Changes detected. Staging..."
git add .

# Commit
echo "💾 Committing: $COMMIT_MSG"
git commit -m "$COMMIT_MSG"

# Push
echo "🚀 Pushing to origin/$BRANCH..."
if git push origin "$BRANCH"; then
    echo "✅ Sync Complete!"
else
    echo "❌ Push Failed. Check credentials."
    exit 1
fi
