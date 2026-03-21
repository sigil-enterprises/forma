# Your First Document

A worked example: creating a proposal for a fictional client, **Meridian Logistics**.

## 1. Scaffold

```bash
forma init meridian-logistics
```

## 2. Add your logo

Place `logo.png` and `logo-white.png` in `documents/meridian-logistics/assets/`.

## 3. Set brand colors

Edit `documents/meridian-logistics/style.yaml`:

```yaml
brand:
  logo: assets/logo.png
  logo_white: assets/logo-white.png

colors:
  primary_dark:   "#1A2B3C"
  primary_accent: "#E84B3C"

typography:
  font_primary:   "Rubik"
  font_secondary: "Inter"
```

## 4. Draft content

Write brief meeting notes to `notes.md`:

```markdown
# Meridian Logistics — Discovery Call

Client: Meridian Logistics (freight, ~1200 staff)
Contact: Sara Chen, VP Operations, sara@meridian.co

Problem:
- Manual freight tracking across 6 legacy systems
- 4-hour average delay between booking and visibility
- Dispatchers spend 60% of time on phone calls

Proposed approach:
- Unified API gateway across all freight systems
- Real-time tracking dashboard
- AI-assisted dispatch recommendations

Timeline: 6 months, 3 phases
Investment: ~$220,000
```

Then:

```bash
forma compose fill documents/meridian-logistics \
  --notes notes.md
```

Claude drafts the full `content.yaml` from your notes.

## 5. Review and fill gaps

Open `documents/meridian-logistics/content.yaml`. Fields without data are marked `TODO:` — replace them with real information.

## 6. Validate and render

```bash
forma validate documents/meridian-logistics
forma render documents/meridian-logistics
```

## 7. Iterate

With `--watch`, forma re-renders every time you save `content.yaml` or `style.yaml`:

```bash
forma render documents/meridian-logistics --watch
```

Open `var/builds/meridian-logistics/slides.pdf` in your PDF viewer with auto-reload enabled.
