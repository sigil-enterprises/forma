# Quick Start

This guide gets you from zero to a rendered PDF in five minutes.

## 1. Scaffold a project

```bash
forma init acme-corp
```

This creates:

```
documents/acme-corp/
├── forma.yaml        # schema, templates, output config
├── content.yaml      # skeleton content (fill this in)
├── style.yaml        # visual tokens (colors, fonts)
└── assets/           # place logos and images here
```

## 2. Fill content

### Option A — Write by hand

Edit `documents/acme-corp/content.yaml` directly. The skeleton has `TODO:` placeholders for every required field.

### Option B — Use Claude

```bash
forma compose fill documents/acme-corp \
  --notes meeting-notes.md
```

Claude reads your notes and drafts a complete `content.yaml` validated against the schema. See [Claude Composer](../guide/composer.md).

## 3. Validate

```bash
forma validate documents/acme-corp
```

```
✓ Validation passed
```

If there are errors, forma prints them with field paths:

```
✗ 2 validation error(s)
  • client.contact.email: field required
  • investment.phases[0].items[0].rate_usd: value is not a valid float
```

## 4. Render

```bash
forma render documents/acme-corp
```

Output PDFs land in `var/builds/acme-corp/`:

```
var/builds/acme-corp/
├── slides.pdf
└── report.pdf
```

Render a single template:

```bash
forma render documents/acme-corp --template slides
```

Watch mode (re-renders on save):

```bash
forma render documents/acme-corp --watch
```

## 5. Publish

```bash
forma publish documents/acme-corp
```

Renders all templates then uploads each PDF to the Google Drive folder configured in `forma.yaml`. Requires `GOOGLE_SERVICE_ACCOUNT_JSON` to be set.

Dry run (skips the upload):

```bash
forma publish documents/acme-corp --dry-run
```
