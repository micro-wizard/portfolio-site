# nathanspelts.com

A personal site: resume on the front page, markdown blog behind it, gruvbox in
light and dark. The generator is a small Rust program in `src/` whose only
dependency is a CommonMark parser, everything else is the standard library.
The same resume markdown also typesets the PDF resume through LaTeX.

```
cargo run -- build        # render content/ into dist/
cargo run -- serve        # build, serve on :8080, rebuild on change
cargo run -- serve 3000   # …on another port
SITE_DRAFTS=1 cargo run -- build   # include posts marked draft: true
```

## Layout

```
site.toml            title, author, base_url, domain
content/index.md     the resume front page and PDF
content/blog/*.md    posts        -> /blog/<slug>/
content/pages/*.md   flat pages   -> /<slug>/
templates/*.html     {{ name }} substitution, no template language
static/              copied verbatim to the site root (CSS lives here)
resume/              LaTeX template, class and fonts for the PDF resume
resume/variants/     company-tailored resumes, gitignored
include/particle-simulator/
                     git submodule; its src/ is served at /particles/sim/
dist/                build output, gitignored
```

## Writing

**A post** is `content/blog/2026-02-11-my-title.md`. The date prefix sets the
post date and is stripped from the URL, so that file becomes `/blog/my-title/`.

```markdown
---
title: My title
date: 2026-02-11
summary: One sentence; shown in the index and the feed.
tags: rust, web
draft: false
---

Body in CommonMark. Tables, footnotes, strikethrough and task lists are on.
```

Every front-matter field is optional except the date (filename or `date:`).
`slug:` overrides the URL.

**The resume** is `content/index.md`. The same file renders the front page
and the LaTeX PDF (see [Resume PDF](#resume-pdf)). Its front matter holds the
header. `link:` may repeat, and a third field keeps a link to one output:

```markdown
---
name: Nathan Spelts
tagline: Shown under the name on the site.
pdf_title: Nathan Spelts' CV
public: true                     # see "Contact details" below
location: Vancouver, WA          # PDF only
email: nathan at spelts dot net
link: GitHub | https://github.com/micro-wizard`
link: Resume (PDF) | /file/nathanSpeltsResume.pdf | web
---
```

The body is markdown with a little structure. `##` starts a section. Text
before the first `###` in a section belongs to the section itself: the
summary paragraph, or the skills list. `###` starts an entry, and the line
under it is split on ` · `:

```markdown
## Work Experience

### [Company](https://company.com)

Job Title · City, ST · Aug 2023 – Present

- What you did, one bullet per line.

## Projects {.web}

### [Project](https://github.com/you/project)

One-line overview.

- Detail.

## Skills

- **Languages:** Rust, C, Python
```

With two or more parts, the meta line is a job or degree: the last part is
the date range (right-aligned), the first is the role, and anything between
is the location. With one part, it's a project's overview, and the PDF shows
the heading's link on the right. Add `{.web}` or `{.pdf}` after a `##` or
`###` title to keep it out of the other output.

**Contact details.** `email:` can be written plainly or as
`you at example dot com`. The site only ever shows it spelled out that way,
as plain text with no mailto link, so a person has to type it in. `phone:`
only goes in the PDF. `public: true` marks a resume that is linked from the
site: its PDF leaves the phone out even if one is set, and spells the email
out the same way. The company variants in `resume/variants/` leave `public`
off, so they keep the plain email and phone. In a post, write addresses out
the same way by hand.

The front page also has print styles: **Print → Save as PDF** gives you a
clean copy with the site chrome stripped out.

## Resume PDF

```
cargo run -- resume              # content/index.md -> resume/build/, and
                                 # copies it to static/file/nathanSpeltsResume.pdf
cargo run -- resume garmin       # resume/variants/garmin.md
cargo run -- resume --typ garmin # write the .typ only, skip typst
make resume VARIANT=garmin       # same, then open the PDF
```

This needs `typst` on the path (`brew install typst`). The site build
doesn't: the published PDF is committed under `static/`, so rerun
`cargo run -- resume` and commit when the resume changes.

`resume/resume.typ` is the document template, `resume/lib.typ` is the layout
it imports, and `resume/Fonts/` holds the Tinos faces it sets. Company-tailored versions are
full copies of `content/index.md` in `resume/variants/`, which is gitignored.
PDFs are named `nathan_spelts_<variant>_resume.pdf`.

## Colour scheme

Gruvbox light and dark, defined as CSS variables at the top of
`static/style.css`. The site follows the OS setting until the reader clicks the
toggle, which is then remembered in `localStorage`.

## Hosting on GitHub Pages with your own domain

1. Push this repo to GitHub as `main`.
2. **Settings → Pages → Source: GitHub Actions.** The workflow in
   `.github/workflows/deploy.yml` builds and deploys on every push to `main`.
3. Put your domain in `site.toml`:

   ```toml
   base_url = "https://yourdomain.com"
   domain   = "yourdomain.com"
   ```

   `domain` is written to `dist/CNAME` on every build, which is what Pages
   reads to serve the custom domain.
4. At your DNS provider, point the domain at GitHub:

   | Record | Name | Value |
   | --- | --- | --- |
   | A | `@` | `185.199.108.153` |
   | A | `@` | `185.199.109.153` |
   | A | `@` | `185.199.110.153` |
   | A | `@` | `185.199.111.153` |
   | CNAME | `www` | `yourname.github.io` |

   For a subdomain only (`www.yourdomain.com`), skip the A records and use the
   CNAME alone.
5. **Settings → Pages → Custom domain**: enter the domain, then tick **Enforce
   HTTPS** once the certificate is issued (a few minutes).

Hosting it as a project page (`yourname.github.io/repo`) instead works too,
set `base_path = "/repo/"` in `site.toml` and leave `domain` empty.

## Notes

- `dist/` is disposable; the build wipes and re-creates it each time.
- `.nojekyll` is emitted so Pages serves the tree untouched.
- `feed.xml` (Atom), `sitemap.xml`, `robots.txt` and a `404.html` are generated.
- No syntax highlighting: it would mean a much larger dependency. Code blocks
  are styled, just not coloured.
