# Secrets & Environment

## .env file

Copy `.env.example` to `.env` and fill in your values:

```bash
cp .env.example .env
```

`.env` is loaded automatically by the devcontainer. Never commit it to version control.

## Variable reference

### Required for composing

| Variable | Description |
|---|---|
| `ANTHROPIC_API_KEY` | Claude API key. Get one at [console.anthropic.com](https://console.anthropic.com). Required for `forma compose fill` and `forma compose enrich`. |

### Required for publishing

| Variable | Description |
|---|---|
| `GOOGLE_SERVICE_ACCOUNT_JSON` | Base64-encoded Google service account JSON. Required for `forma publish` and the `google_docs` / `google_sheets` skills. See [Publishing](../guide/publishing.md). |

### Skills

| Variable | Skill | Description |
|---|---|---|
| `GOOGLE_DOCS_ID` | `google_docs` | Google Docs document ID |
| `GOOGLE_SHEETS_ID` | `google_sheets` | Google Sheets spreadsheet ID |
| `GOOGLE_SHEETS_RANGE` | `google_sheets` | Sheet range (default: `Sheet1!A1:Z`) |
| `CLICKUP_API_TOKEN` | `clickup` | ClickUp API token |
| `CLICKUP_LIST_ID` | `clickup` | ClickUp list ID to fetch tasks from |
| `MEETING_NOTES_PATH` | `meeting_notes` | Path to meeting notes markdown file |

### CI / devcontainer

| Variable | Description |
|---|---|
| `GITHUB_TOKEN` | GitHub personal access token (repo read scope). Used by `setup-host` and CI workflows. |
| `ORGANIZATION` | GitHub organization name. Set automatically by `setup-host`. |
| `DEV_CONTAINER_NAME` | Container name (`org/project`). Set automatically by `setup-host`. |

## GitHub Actions secrets

Set these in **Settings → Secrets and variables → Actions**:

| Secret | Required for |
|---|---|
| `PAT_ORG_REPO_READ` | Checking out the repository and submodules in CI |
| `PAT_BADGES` | Writing coverage badges to the repo |
| `GOOGLE_SERVICE_ACCOUNT_JSON` | `publish.yml` Drive uploads |
| `ANTHROPIC_API_KEY` | Optional: only if running `forma compose` in CI |

## Encoding the service account

```bash
# Encode
openssl base64 -in service-account.json | tr -d '\n'

# Decode and verify (should print valid JSON)
echo "$GOOGLE_SERVICE_ACCOUNT_JSON" | base64 -d | python3 -m json.tool
```

The credential is decoded in memory in `publisher/google_drive.py` — it never touches the filesystem.
