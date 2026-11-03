//! The resume: one markdown file rendered two ways — HTML for the front page
//! and Typst (resume/lib.typ) for the PDF.
//!
//! The body is ordinary markdown with a little structure:
//!
//!   ## Work Experience            a section; add `{.web}` or `{.pdf}` after
//!                                 the title to keep it out of the other output
//!   Paragraphs and lists before the first `###` belong to the section
//!   itself (a summary, a skills list).
//!
//!   ### [Company](https://url)    an entry; the link is optional
//!   Role · Location · Dates       optional meta line, split on " · "
//!
//!   - bullet points
//!
//! A meta line with two or more parts is a job or degree: the last part is
//! the date range. With one part it is a project's one-line overview.
//!
//! Contact details: `email:` may be written plainly or as
//! `name at domain dot com`. The site only ever shows it spelled out that
//! way, as text rather than a link, so a person has to type it in. `phone:`
//! only appears in the PDF. A file marked `public: true` (the front page)
//! prints no phone and spells the email out in its PDF too, since that PDF is
//! linked from the site.
//!
//! A cover letter (resume/letters/<variant>.md, plain markdown paragraphs) is
//! typeset under the same letterhead as its variant's resume, by
//! resume/cover-letter.typ.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

use crate::config::Config;
use crate::frontmatter::Doc;
use crate::template::{escape, Template};

const DIR: &str = "resume";
const VARIANTS: &str = "resume/variants";
const LETTERS: &str = "resume/letters";
const OUT: &str = "resume/build";

#[derive(Clone, Copy, PartialEq)]
pub enum Only {
    Both,
    Web,
    Pdf,
}

impl Only {
    fn web(self) -> bool {
        self != Only::Pdf
    }
    fn pdf(self) -> bool {
        self != Only::Web
    }
}

pub struct Link {
    pub label: String,
    pub href: String,
    only: Only,
}

struct Section {
    title: String,
    only: Only,
    intro: String,
    entries: Vec<Entry>,
}

struct Entry {
    title: String,
    url: String,
    only: Only,
    meta: Vec<String>,
    body: String,
}

pub struct Resume {
    pub doc: Doc,
    email: String,
    public: bool,
    links: Vec<Link>,
    sections: Vec<Section>,
}

impl Resume {
    pub fn load(path: &Path) -> io::Result<Resume> {
        let src = fs::read_to_string(path).map_err(|e| {
            io::Error::new(e.kind(), format!("cannot read {}: {e}", path.display()))
        })?;
        Ok(Resume::parse(Doc::parse(&src)))
    }

    fn parse(doc: Doc) -> Resume {
        // `link: Label | href` with an optional third field, `web` or `pdf`.
        let links = doc
            .all("link")
            .iter()
            .map(|entry| {
                let mut parts = entry.split('|').map(str::trim);
                let label = parts.next().unwrap_or("").to_string();
                let href = parts.next().unwrap_or(&label).to_string();
                let only = match parts.next() {
                    Some("web") => Only::Web,
                    Some("pdf") => Only::Pdf,
                    _ => Only::Both,
                };
                Link { label, href, only }
            })
            .collect();
        let sections = parse_sections(&doc.body);
        let email = unmask_email(doc.get("email"));
        let public = doc.flag("public");
        Resume { doc, email, public, links, sections }
    }

    /// The email line under the tagline: `nathan at spelts dot net`, with
    /// the spelled-out words marked so they can be styled apart.
    pub fn web_email(&self) -> String {
        let Some((user, domain)) = self.email.split_once('@') else {
            return String::new();
        };
        let domain = domain
            .split('.')
            .map(escape)
            .collect::<Vec<_>>()
            .join(" <span class=\"spelled\">dot</span> ");
        format!(
            "{} <span class=\"spelled\">at</span> {domain}",
            escape(user)
        )
    }

    /// The contact links under the name on the front page.
    pub fn web_links(&self) -> String {
        let mut out = Vec::new();
        for link in self.links.iter().filter(|l| l.only.web()) {
            out.push(format!(
                "<a href=\"{}\">{}<span>{}</span></a>",
                escape(&link.href),
                link_icon(&link.href),
                escape(&link.label)
            ));
        }
        out.join("\n")
    }

    /// The body for the front page; the masthead comes from the template.
    pub fn html(&self) -> String {
        let mut out = String::new();
        for section in self.sections.iter().filter(|s| s.only.web()) {
            if !section.title.is_empty() {
                out.push_str(&format!("<h2>{}</h2>\n", inline_html(&section.title)));
            }
            out.push_str(&crate::markdown(&section.intro));
            for entry in section.entries.iter().filter(|e| e.only.web()) {
                let mut title = inline_html(&entry.title);
                if !entry.url.is_empty() {
                    title = format!("<a href=\"{}\">{title}</a>", escape(&entry.url));
                }
                let (lead, dates) = match entry.meta.as_slice() {
                    [] => (&[][..], None),
                    [only] => (std::slice::from_ref(only), None),
                    [lead @ .., dates] => (lead, Some(dates)),
                };
                let when = dates
                    .map(|d| format!(" <span class=\"when\">{}</span>", inline_html(d)))
                    .unwrap_or_default();
                out.push_str(&format!("<h3>{title}{when}</h3>\n"));
                if !lead.is_empty() {
                    let lead: Vec<String> = lead.iter().map(|m| inline_html(m)).collect();
                    out.push_str(&format!("<p>{}</p>\n", lead.join(" · ")));
                }
                out.push_str(&crate::markdown(&entry.body));
            }
        }
        out
    }

    /// A complete .typ document for resume/resume.typ.
    pub fn typst(&self, template: &Template, website: &str) -> String {
        let mut body = String::new();
        for section in self.sections.iter().filter(|s| s.only.pdf()) {
            let entries: Vec<&Entry> = section.entries.iter().filter(|e| e.only.pdf()).collect();
            if section.intro.trim().is_empty() && entries.is_empty() {
                continue;
            }
            if !body.is_empty() {
                body.push_str("#section-gap\n");
            }
            if !section.title.is_empty() {
                body.push_str(&format!("#section[{}]\n", typst(&section.title, INLINE)));
            }
            let intro = typst(&section.intro, BLOCK);
            if !intro.is_empty() {
                body.push_str(&intro);
                body.push_str("\n\n");
            }
            for entry in entries {
                body.push_str(&typst_entry(entry));
            }
        }

        let doc = &self.doc;
        template.render(&[
            ("author", &typst_string(doc.get("name"))),
            ("title", &typst_string(doc.get("pdf_title"))),
            ("name", &typst(doc.get("name"), INLINE)),
            ("subtitle", &self.typst_subtitle(website, self.public)),
            ("body", &body),
        ])
    }

    /// A complete .typ document for resume/cover-letter.typ: `letter`, in
    /// markdown, under this resume's name and contact details. Those are
    /// never the public ones: a letter goes to one employer, not the site.
    pub fn typst_letter(&self, template: &Template, website: &str, letter: &str) -> String {
        let name = self.doc.get("name");
        template.render(&[
            ("author", &typst_string(name)),
            ("title", &typst_string(&format!("{name}: Cover Letter"))),
            ("name", &typst(name, INLINE)),
            ("subtitle", &self.typst_subtitle(website, false)),
            // A letter's lines are its own: `Sincerely,` and the name under
            // it stay on two lines without a blank one between them.
            ("body", &typst_with(letter, BULLETS, "\\\n")),
        ])
    }

    /// Phone · email · location, then the remaining links on their own line.
    /// `public` leaves the phone out and spells the email out.
    fn typst_subtitle(&self, website: &str, public: bool) -> String {
        const SEP: &str = " #sub-sep ";
        let pdf_links: Vec<&Link> = self.links.iter().filter(|l| l.only.pdf()).collect();
        let mut first = Vec::new();
        let phone = self.doc.get("phone");
        if !phone.is_empty() && !public {
            first.push(typst(phone, INLINE));
        }
        if public && !self.email.is_empty() {
            first.push(typst(&spell_email(&self.email), INLINE));
        } else if !self.email.is_empty() {
            let email = &self.email;
            first.push(format!(
                "#link(\"mailto:{}\")[{}]",
                typst_string(email),
                typst(email, INLINE)
            ));
        }
        let location = self.doc.get("location");
        if !location.is_empty() {
            first.push(typst(location, INLINE));
        }
        let second: Vec<String> = pdf_links.iter().map(|l| shown_link(l.href(website))).collect();
        let mut out = String::from("#subtitle[\n");
        out.push_str(&first.join(SEP));
        if !second.is_empty() {
            out.push_str("\n\n");
            out.push_str(&second.join(SEP));
        }
        out.push_str("\n]");
        out
    }
}

impl Link {
    /// A site-relative href is spelled out in full in the PDF, which is read
    /// away from the site.
    fn href(&self, website: &str) -> String {
        if self.href.starts_with('/') {
            format!("{website}{}", self.href)
        } else {
            self.href.clone()
        }
    }
}

/// A small mark for a contact link on the site, chosen from where it points,
/// or nothing for anywhere else. The brand marks are from Simple Icons (CC0);
/// the document is drawn to match the email icon in templates/resume.html.
fn link_icon(href: &str) -> &'static str {
    let host = href
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.")
        .split('/')
        .next()
        .unwrap_or("");
    match host {
        "github.com" => r#"<svg class="brand" viewBox="0 0 24 24" aria-hidden="true"><path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>"#,
        "linkedin.com" => r#"<svg class="brand" viewBox="0 0 24 24" aria-hidden="true"><path d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>"#,
        _ if href.ends_with(".pdf") => {
            r#"<svg class="doc" viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 1.5h6l3 3v10h-9zM9.5 1.5v3h3M5.5 8h5M5.5 10.5h5M5.5 13h3"/></svg>"#
        }
        _ => "",
    }
}

/// A blue link showing the bare URL, the way the subtitle and the projects
/// print them.
fn shown_link(href: String) -> String {
    let shown = href
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    format!("#shown-link(\"{}\")[{}]", typst_string(&href), typst(shown, INLINE))
}

/// `nathan at spelts dot net` -> `nathan@spelts.net`; plain addresses
/// pass through.
fn unmask_email(src: &str) -> String {
    src.trim().replace(" at ", "@").replace(" dot ", ".")
}

/// `nathan@spelts.net` -> `nathan at spelts dot net`.
pub fn spell_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => format!("{user} at {}", domain.replace('.', " dot ")),
        None => email.to_string(),
    }
}

fn parse_sections(body: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = Vec::new();
    let new_section = |title: &str| {
        let (title, only) = split_attrs(title);
        Section { title, only, intro: String::new(), entries: Vec::new() }
    };
    for line in body.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            sections.push(new_section(heading));
            continue;
        }
        if sections.is_empty() {
            // Anything above the first `##` becomes an untitled section.
            sections.push(new_section(""));
        }
        let section = sections.last_mut().unwrap();
        if let Some(heading) = line.strip_prefix("### ") {
            let (heading, only) = split_attrs(heading);
            let (title, url) = split_link(&heading);
            section.entries.push(Entry { title, url, only, meta: Vec::new(), body: String::new() });
        } else {
            let target = match section.entries.last_mut() {
                Some(entry) => &mut entry.body,
                None => &mut section.intro,
            };
            target.push_str(line);
            target.push('\n');
        }
    }
    for entry in sections.iter_mut().flat_map(|s| s.entries.iter_mut()) {
        split_meta(entry);
    }
    sections
}

/// Pulls the `Role · Location · Dates` line off the front of an entry body.
fn split_meta(entry: &mut Entry) {
    let body = entry.body.trim_start_matches(['\n', '\r']);
    let end = body.find("\n\n").unwrap_or(body.len());
    let first = body[..end].trim();
    let is_block = first.is_empty()
        || first.starts_with(['-', '*', '+', '<', '>', '#', '|', '`'])
        || first.split_once(". ").is_some_and(|(n, _)| n.chars().all(|c| c.is_ascii_digit()));
    if is_block {
        return;
    }
    entry.meta = first
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .split(" · ")
        .map(|part| part.trim().to_string())
        .collect();
    entry.body = body[end..].to_string();
}

/// `Projects {.web}` -> ("Projects", Only::Web).
fn split_attrs(heading: &str) -> (String, Only) {
    let heading = heading.trim();
    if let Some(open) = heading.rfind(" {") {
        if heading.ends_with('}') {
            let only = match heading[open + 2..heading.len() - 1].trim() {
                ".web" => Some(Only::Web),
                ".pdf" => Some(Only::Pdf),
                _ => None,
            };
            if let Some(only) = only {
                return (heading[..open].trim_end().to_string(), only);
            }
        }
    }
    (heading.to_string(), Only::Both)
}

/// `[Company](https://url)` -> ("Company", "https://url").
fn split_link(heading: &str) -> (String, String) {
    if let Some(inner) = heading.strip_prefix('[').and_then(|h| h.strip_suffix(')')) {
        if let Some((text, url)) = inner.split_once("](") {
            return (text.to_string(), url.trim().to_string());
        }
    }
    (heading.to_string(), String::new())
}

/// Markdown for a single line, without the wrapping `<p>`.
fn inline_html(src: &str) -> String {
    let html = crate::markdown(src);
    let html = html.trim();
    html.strip_prefix("<p>")
        .and_then(|h| h.strip_suffix("</p>"))
        .unwrap_or(html)
        .to_string()
}

fn typst_entry(entry: &Entry) -> String {
    let title = typst(&entry.title, INLINE);
    let mut head = if entry.url.is_empty() {
        format!("*{title}*")
    } else {
        format!("#link(\"{}\")[*{title}*]", typst_string(&entry.url))
    };
    let (lead, dates) = match entry.meta.as_slice() {
        [] => (None, None),
        [only] => (Some(only), None),
        [role, rest @ ..] => (Some(role), Some(rest)),
    };
    let aside = match dates {
        // A job or degree: the heading links the organisation, the places
        // follow it and the dates sit on the right.
        Some(rest) => {
            let (places, dates) = rest.split_at(rest.len() - 1);
            for place in places {
                head.push_str(&format!(", {}", typst(place, INLINE)));
            }
            Some(typst_dates(&dates[0]))
        }
        // A project: its link is spelled out on the right instead.
        None if !entry.url.is_empty() => Some(shown_link(entry.url.clone())),
        None => None,
    };

    let mut out = format!("#entry([{head}]");
    if let Some(aside) = aside {
        out.push_str(&format!(", aside: [{aside}]"));
    }
    if let Some(lead) = lead {
        out.push_str(&format!(", role: [{}]", typst(lead, INLINE)));
    }
    let detail = typst(&entry.body, BULLETS);
    if !detail.is_empty() {
        out.push_str(&format!(", detail: [\n{detail}\n]"));
    }
    out.push_str(")\n");
    out
}

/// `Aug 2023 – Present` -> `#box[Aug 2023] -- #box[Present]`, so that neither
/// end of the range is broken across lines.
fn typst_dates(src: &str) -> String {
    typst(src, INLINE)
        .split(['–', '—'])
        .map(|part| format!("#box[{}]", part.trim()))
        .collect::<Vec<_>>()
        .join(" -- ")
}

/// How the items of a markdown list are wrapped in the Typst output.
type Wrap = (&'static str, &'static str);

/// For a fragment that holds no list: a heading, a date, a line of meta.
const INLINE: Wrap = ("", "");
/// One flush-left paragraph per item, which is what the skills list wants.
const BLOCK: Wrap = ("#block[", "]\n");
/// A dotted bullet per item, under a job or a project.
const BULLETS: Wrap = ("#sub-bullet[", "]\n");

/// Markdown to Typst markup. List items are wrapped in `item`; nested lists
/// flatten, and inline HTML is dropped.
fn typst(src: &str, item: Wrap) -> String {
    typst_with(src, item, "\n")
}

/// As `typst`, with a line break within a paragraph written as `soft`.
fn typst_with(src: &str, item: Wrap, soft: &str) -> String {
    let mut out = String::new();
    let mut in_item = 0;
    let mut in_code = false;
    // In a URL written out in the text, which may go on in the next chunk.
    let mut url = false;
    for event in Parser::new_ext(src, crate::markdown_options()) {
        if !matches!(event, Event::Text(_) | Event::Code(_)) {
            url = false;
        }
        match event {
            Event::Start(Tag::Item) => {
                in_item += 1;
                out.push_str(item.0);
                out.push(OPEN);
            }
            Event::End(TagEnd::Item) => {
                in_item -= 1;
                out.push_str(item.1);
            }
            // A heading, code block or table isn't set as one, but stays a
            // paragraph of its own: `Best regards,` underlined with `--` is
            // a heading, not part of the name under it.
            Event::End(TagEnd::Paragraph | TagEnd::Heading(_) | TagEnd::CodeBlock | TagEnd::Table)
                if in_item == 0 =>
            {
                in_code = false;
                out.push_str("\n\n")
            }
            Event::End(TagEnd::CodeBlock) => in_code = false,
            Event::End(TagEnd::TableCell) => out.push(' '),
            Event::End(TagEnd::TableHead | TagEnd::TableRow) => out.push_str("\\\n"),
            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => out.push('*'),
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => out.push('_'),
            Event::Start(Tag::Link { dest_url, .. }) => {
                out.push_str(&format!("#link(\"{}\")[{OPEN}", typst_string(&dest_url)));
            }
            Event::End(TagEnd::Link) => out.push(']'),
            // The class set code in the body face, and so does this.
            Event::Start(Tag::CodeBlock(_)) => in_code = true,
            // A code block's lines stay apart, as a paragraph's do.
            Event::Text(text) if in_code => {
                let text = typst_escape(&text, &mut url);
                let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
                out.push_str(&lines.join(soft));
            }
            Event::Text(text) | Event::Code(text) => out.push_str(&typst_escape(&text, &mut url)),
            Event::SoftBreak => out.push_str(soft),
            Event::HardBreak => out.push_str("\\\n"),
            _ => {}
        }
    }
    // Line by line, and from each `[` this wrote, whatever chunks the
    // parser handed the text over in.
    let lines: Vec<String> = out
        .trim()
        .split('\n')
        .map(|line| line.split(OPEN).map(escape_line_start).collect())
        .collect();
    lines.join("\n")
}

/// Marks where `typst_with` opened a content block, which Typst reads like
/// the start of a line. Text can't hold it: it's a control character.
const OPEN: char = '\u{1}';

/// Everything Typst would read as markup rather than as text. Dashes, quotes
/// and `...` are left alone: Typst turns them into the same dashes, curly
/// quotes and ellipses that the TeX ligatures did. Every `/` is escaped, as
/// `//` and `/*` start comments, wherever the text around it came from;
/// except in a URL written out in the text, from `http://` or `https://` to
/// the next space, which Typst links as a whole. `url` says whether the text
/// before this went into one.
fn typst_escape(text: &str, url: &mut bool) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, c) in text.char_indices() {
        if c.is_whitespace() {
            *url = false;
        } else if text[i..].starts_with("http://") || text[i..].starts_with("https://") {
            *url = true;
        }
        if "\\#$*_[]<>@~`".contains(c) || (c == '/' && !*url) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// A line of Typst output, with what Typst would read at its start as a
/// heading or list (`= `, `== `, `- `, `+ `, `2. `, each followed by a space
/// or the end of the line) escaped. Anything else is left alone, so that
/// `--`, `-5` and `2.5` keep their dashes and numbers. Markdown has already
/// taken its own lists and headings, so these are plain text, such as a
/// letter's line that opens with a number. `typst_with` also calls it on
/// the text that opens a bullet or a link's label. The markup it writes
/// starts with `#`, `\`, `*`, `_` or `]`, so it's never touched.
fn escape_line_start(line: &str) -> String {
    // Typst reads a marker after an indent as well.
    let text = line.trim_start();
    let indent = &line[..line.len() - text.len()];
    let digits = text.len() - text.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    let marker = if text.starts_with('=') {
        text.len() - text.trim_start_matches('=').len()
    } else if text.starts_with(['-', '+']) {
        1
    } else if digits > 0 && text[digits..].starts_with('.') {
        digits + 1
    } else {
        return line.to_string();
    };
    if !text[marker..].is_empty() && !text[marker..].starts_with(char::is_whitespace) {
        line.to_string()
    } else if digits > 0 {
        format!("{indent}{}\\{}", &text[..digits], &text[digits..])
    } else {
        format!("{indent}\\{text}")
    }
}

/// Text inside a Typst string literal, as a URL or a document property.
fn typst_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// `site resume [--typ] [variant ...]`: write the .typ for each variant and,
/// unless `--typ` is given, typeset it with typst. The default variant is
/// content/index.md, the same file as the front page, and its PDF is copied
/// into static/ for the site to serve.
pub fn command(args: &[String]) -> io::Result<()> {
    let cfg = Config::load("site.toml");
    let template = Template::load(DIR, "resume.typ");
    each_variant(args, |name, resume, typ_only| {
        let job = format!("{}_{name}_resume", slug(resume.doc.get("name")));
        let Some(pdf) = write(&job, &resume.typst(&template, &cfg.base_url()), typ_only)? else {
            return Ok(());
        };
        let publish = cfg.get("resume_pdf");
        if name == "default" && !publish.is_empty() {
            let target = Path::new("static").join(publish);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&pdf, &target)?;
            println!("copied to {}", target.display());
        }
        Ok(())
    })
}

/// `site cover-letter [--typ] [variant ...]`: as `site resume`, for the
/// letter in resume/letters/<variant>.md, with the name and contact details
/// of the variant's resume.
pub fn letter_command(args: &[String]) -> io::Result<()> {
    let cfg = Config::load("site.toml");
    let template = Template::load(DIR, "cover-letter.typ");
    each_variant(args, |name, resume, typ_only| {
        let path = Path::new(LETTERS).join(format!("{name}.md"));
        let letter = fs::read_to_string(&path).map_err(|e| {
            io::Error::new(e.kind(), format!("cannot read {}: {e}", path.display()))
        })?;
        let job = format!("{}_{name}_cover_letter", slug(resume.doc.get("name")));
        write(&job, &resume.typst_letter(&template, &cfg.base_url(), &letter), typ_only)?;
        Ok(())
    })
}

/// Loads the resume of each variant named in `args` (the default if none),
/// and hands it to `f` with its name and whether `--typ` was given.
fn each_variant(
    args: &[String],
    mut f: impl FnMut(&str, &Resume, bool) -> io::Result<()>,
) -> io::Result<()> {
    let typ_only = args.iter().any(|a| a == "--typ");
    let mut variants: Vec<&str> = args.iter().map(String::as_str).filter(|a| !a.starts_with("--")).collect();
    if variants.is_empty() {
        variants.push("default");
    }
    fs::create_dir_all(OUT)?;
    for variant in variants {
        let (name, path) = locate(variant);
        f(&name, &Resume::load(&path)?, typ_only)?;
    }
    Ok(())
}

/// Writes resume/build/<job>.typ and, unless `typ_only`, typesets it into
/// the PDF beside it, whose path it returns.
fn write(job: &str, doc: &str, typ_only: bool) -> io::Result<Option<PathBuf>> {
    let typ = Path::new(OUT).join(format!("{job}.typ"));
    fs::write(&typ, doc)?;
    if typ_only {
        println!("wrote {}", typ.display());
        return Ok(None);
    }

    // resume/ is the root: lib.typ is imported as `/lib.typ`, and Tinos
    // is loaded from ./Fonts rather than from the system.
    let status = Command::new("typst")
        .current_dir(DIR)
        .args(["compile", "--root", ".", "--font-path", "Fonts"])
        .arg(format!("build/{job}.typ"))
        .arg(format!("build/{job}.pdf"))
        .status()
        .map_err(|e| io::Error::new(e.kind(), format!("cannot run typst: {e}")))?;
    if !status.success() {
        return Err(io::Error::other(format!("typst failed on {}", typ.display())));
    }
    let pdf = Path::new(OUT).join(format!("{job}.pdf"));
    println!("wrote {}", pdf.display());
    Ok(Some(pdf))
}

/// `default` is the front page; a bare name is resume/variants/<name>.md; a
/// path is used as given.
fn locate(variant: &str) -> (String, PathBuf) {
    if variant == "default" {
        return ("default".into(), PathBuf::from("content/index.md"));
    }
    let path = if variant.ends_with(".md") {
        PathBuf::from(variant)
    } else {
        Path::new(VARIANTS).join(format!("{variant}.md"))
    };
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let name = if stem == "index" { "default".to_string() } else { stem.to_string() };
    (name, path)
}

/// `Nathan Spelts` -> `nathan_spelts`.
fn slug(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    if words.is_empty() { "resume".into() } else { words.join("_") }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "---\nname: Pat Doe\npublic: true\nemail: pat at doe dot net\nphone: 555 0100\n---\n";

    #[test]
    fn a_letter_keeps_its_lines_and_the_resume_joins_them() {
        let src = "Dear Hiring Manager,\n\nSincerely,\nPat Doe\n";
        assert_eq!(
            typst_with(src, BULLETS, "\\\n"),
            "Dear Hiring Manager,\n\nSincerely,\\\nPat Doe"
        );
        assert_eq!(typst(src, BULLETS), "Dear Hiring Manager,\n\nSincerely,\nPat Doe");
    }

    #[test]
    fn a_line_that_looks_like_typst_markup_stays_text() {
        // Markdown takes `- a` as its own list; in a code span it's text.
        let src = "Sincerely,\n2. Pat\n= 3\n== 4\n`- a`\n/ c: d\n10.5 years, - not a list\n\
                   -5 or +1\n==x";
        assert_eq!(
            typst_with(src, BULLETS, "\\\n"),
            "Sincerely,\\\n2\\. Pat\\\n\\= 3\\\n\\== 4\\\n\\- a\\\n\\/ c: d\\\n\
             10.5 years, - not a list\\\n-5 or +1\\\n==x"
        );
        // At the start of a bullet or a link's label, too.
        assert_eq!(
            typst("- `- a`\n- 2\\. b\n\n[= c](https://x)", BULLETS),
            "#sub-bullet[\\- a]\n#sub-bullet[2\\. b]\n#link(\"https://x\")[\\= c]"
        );
        // Typst's own dashes stay dashes.
        assert_eq!(escape_line_start("-- Pat"), "-- Pat");
        assert_eq!(escape_line_start("---"), "---");
        assert_eq!(escape_line_start("-"), "\\-");
    }

    #[test]
    fn a_markdown_escape_doesnt_hide_a_number_from_typst() {
        // pulldown-cmark hands `3\.` over as two chunks of text.
        assert_eq!(typst("3\\. Pat, see https://x.y/z", INLINE), "3\\. Pat, see https://x.y/z");
    }

    #[test]
    fn a_comment_is_escaped_but_a_url_isnt() {
        assert_eq!(
            typst("https://x.y/a//b // c, /* d, e/f", INLINE),
            "https://x.y/a//b \\/\\/ c, \\/\\* d, e\\/f"
        );
        // Typst links only http and https; any other `//` is a comment.
        assert_eq!(typst("ftp://a/b c, HTTPS://A", INLINE), "ftp:\\/\\/a\\/b c, HTTPS:\\/\\/A");
        // However the parser splits the text, and a URL across the split.
        assert_eq!(typst("C/**Rust**", INLINE), "C\\/*Rust*");
        assert_eq!(typst("and/`/etc`", INLINE), "and\\/\\/etc");
        assert_eq!(typst("a/\\/b", INLINE), "a\\/\\/b");
        assert_eq!(typst("https://x.y/a\\_b/c d/e", INLINE), "https://x.y/a\\_b/c d\\/e");
    }

    #[test]
    fn a_sign_off_underlined_as_a_heading_stays_apart_from_the_name() {
        assert_eq!(
            typst_with("Best regards,\n--\nPat Doe", BULLETS, "\\\n"),
            "Best regards,\n\nPat Doe"
        );
        assert_eq!(typst("# Re: a job\n\nDear", INLINE), "Re: a job\n\nDear");
        // A code block's lines stay apart, and an indented marker is text.
        assert_eq!(
            typst_with("```\na\n  - x\n```\n\nDear", BULLETS, "\\\n"),
            "a\\\n  \\- x\n\nDear"
        );
    }

    #[test]
    fn a_letter_has_the_private_contact_details_of_a_public_resume() {
        let resume = Resume::parse(Doc::parse(SRC));
        let template = Template::load(DIR, "cover-letter.typ");
        let letter = resume.typst_letter(&template, "https://x", "Hi");
        assert!(letter.contains("555 0100"), "{letter}");
        assert!(letter.contains("#link(\"mailto:pat@doe.net\")"), "{letter}");
        let pdf = resume.typst_subtitle("https://x", resume.public);
        assert!(!pdf.contains("555"), "{pdf}");
        assert!(pdf.contains("pat at doe dot net"), "{pdf}");
    }

    /// `resume/lib.typ`'s `<resume-fit>` field `key` from `typst query`'s
    /// JSON, which is flat.
    fn fit_field(json: &str, key: &str) -> f64 {
        let at = json.find(&format!("\"{key}\"")).unwrap_or_else(|| panic!("no {key}: {json}"));
        let rest = json[at + key.len() + 2..].trim_start().trim_start_matches(':').trim_start();
        let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')).unwrap_or(rest.len());
        rest[..end].parse().unwrap_or_else(|e| panic!("{key}: {e}: {json}"))
    }

    /// What job-hunt reads to say how much to cut: the front page's resume,
    /// which has to fit on one page, and the same body twice, which can't.
    /// Skipped without typst; CI has it.
    #[test]
    fn the_resume_says_how_long_it_runs() {
        if Command::new("typst").arg("--version").output().is_err() {
            eprintln!("skipped: no typst");
            return;
        }
        let cfg = Config::load("site.toml");
        let template = Template::load(DIR, "resume.typ");
        let src = fs::read_to_string("content/index.md").unwrap();
        let body = src.splitn(3, "---\n").nth(2).unwrap(); // after the front matter
        let twice = format!("{src}\n{body}");
        fs::create_dir_all(OUT).unwrap();
        for (job, text, one_page) in [("fit_test_one", &src, true), ("fit_test_long", &twice, false)] {
            let resume = Resume::parse(Doc::parse(text));
            write(job, &resume.typst(&template, &cfg.base_url()), true).unwrap();
            let out = Command::new("typst")
                .current_dir(DIR)
                .args(["query", "--root", ".", "--font-path", "Fonts"])
                .arg(format!("build/{job}.typ"))
                .args(["<resume-fit>", "--field", "value", "--one"])
                .output()
                .unwrap();
            fs::remove_file(Path::new(OUT).join(format!("{job}.typ"))).unwrap();
            let json = String::from_utf8_lossy(&out.stdout);
            assert!(out.status.success(), "{job}: {}", String::from_utf8_lossy(&out.stderr));
            let (height, room) = (fit_field(&json, "height"), fit_field(&json, "room"));
            let pages = fit_field(&json, "pages");
            // US letter, less the margins: 11in - 0.60in - 0.45in.
            assert!((room - 716.4).abs() < 0.01, "{json}");
            assert_eq!(fit_field(&json, "lead"), 14.5, "{json}");
            if one_page {
                assert_eq!(pages, 1.0, "content/index.md runs past a page: {json}");
                assert!(height > 0.0 && height <= room, "{json}");
            } else {
                assert!(pages >= 2.0, "{json}");
                assert!(height > room, "{json}");
            }
        }
    }
}
