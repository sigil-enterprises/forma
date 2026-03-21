# Publishing

`forma publish` renders all templates to PDF and uploads them to Google Drive.

## Setup

### 1. Create a service account

1. Go to [Google Cloud Console](https://console.cloud.google.com) → IAM & Admin → Service Accounts
2. Create a service account (e.g. `forma-publisher`)
3. Download the JSON key file

### 2. Share your Drive folder

Share the target Google Drive folder with the service account email
(e.g. `forma-publisher@your-project.iam.gserviceaccount.com`) with **Editor** access.

### 3. Encode the credentials

```bash
openssl base64 -in service-account.json | tr -d '\n'
```

Set the output as `GOOGLE_SERVICE_ACCOUNT_JSON` in your `.env` file.
The credentials are decoded in memory and never written to disk.

### 4. Configure forma.yaml

```yaml
publishing:
  google_drive_folder_id: "1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs"  # folder ID from Drive URL
  filename_prefix: "SLVR"
```

The folder ID is the last path component in a Google Drive folder URL.

## Usage

```bash
# Render + publish all templates
forma publish documents/acme-corp

# Render + publish a specific template
forma publish documents/acme-corp --template slides

# Override the Drive folder
forma publish documents/acme-corp --folder-id 1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs

# Dry run (renders but does not upload)
forma publish documents/acme-corp --dry-run
```

Dry run output:

```
DRY RUN: would upload var/builds/acme-corp/slides.pdf → Drive/1BxiMV.../SLVR-slides.pdf
DRY RUN: would upload var/builds/acme-corp/report.pdf → Drive/1BxiMV.../SLVR-report.pdf
```

## File naming

Uploaded files are named `{prefix}-{template}.pdf`. If a file with that name
already exists in the folder, it is updated in place (not duplicated).

| Config | Upload name |
|---|---|
| `filename_prefix: "SLVR"`, template `slides` | `SLVR-slides.pdf` |
| `filename_prefix: ""`, template `report` | `report.pdf` |

## Automated publishing via CI

The `publish.yml` workflow runs on every push to `main`:

1. Discovers all `documents/*/forma.yaml` files
2. For each document: validate → render → publish to Drive
3. Uploads PDFs as GitHub Actions artifacts (regardless of Drive publish result)

```yaml
# .github/workflows/publish.yml (excerpt)
- name: Validate
  run: forma validate ${{ matrix.document }}

- name: Render
  run: forma render ${{ matrix.document }}

- name: Publish to Google Drive
  run: forma publish ${{ matrix.document }}
  continue-on-error: true   # Drive publish is best-effort
```

Required CI secrets: `GOOGLE_SERVICE_ACCOUNT_JSON`.
