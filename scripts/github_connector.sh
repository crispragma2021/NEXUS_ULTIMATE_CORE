#!/bin/bash
# NEXUS_SKILL: GitHub Connector
# Accesses repositories, issues, and PRs via GitHub CLI

COMMAND=$1
ARG1=$2
ARG2=$3

if ! command -v gh &> /dev/null; then
    echo "❌ Error: 'gh' CLI not found. Please install: sudo apt install gh"
    exit 1
fi

if [ -z "$GITHUB_TOKEN" ]; then
    # Try to load from .env if not in environment
    if [ -f ../.env ]; then
        export $(grep -v '^#' ../.env | xargs)
    fi
fi

case "$COMMAND" in
    "list_repos")
        echo "📂 Repositories for current user:"
        gh repo list --limit 10 --json name,description,url --template '{{range .}}{{printf "- %s: %s (%s)\n" .name .description .url}}{{end}}'
        ;;
    "search_issues")
        QUERY="$ARG1"
        echo "search_issues: $QUERY"
        gh search issues "$QUERY" --limit 5 --json title,url --template '{{range .}}{{printf "- %s (%s)\n" .title .url}}{{end}}'
        ;;
    "create_issue")
        REPO="$ARG1"
        TITLE="$ARG2"
        echo "Creating issue in $REPO: $TITLE"
        gh issue create --repo "$REPO" --title "$TITLE" --body "Created by Nexus Alpha"
        ;;
    *)
        echo "Usage: $0 {list_repos|search_issues <query>|create_issue <repo> <title>}"
        exit 1
        ;;
esac
