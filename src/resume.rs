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
            ("subtitle", &self.typst_subtitle(website)),
            ("body", &body),
        ])
    }

    /// Phone · email · location, then the remaining links on their own line.
    fn typst_subtitle(&self, website: &str) -> String {
        const SEP: &str = " #sub-sep ";
        let pdf_links: Vec<&Link> = self.links.iter().filter(|l| l.only.pdf()).collect();
        let mut first = Vec::new();
        let phone = self.doc.get("phone");
        if !phone.is_empty() && !self.public {
            first.push(typst(phone, INLINE));
        }
        if self.public && !self.email.is_empty() {
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
    let mut out = String::new();
    let mut in_item = 0;
    for event in Parser::new_ext(src, crate::markdown_options()) {
        match event {
            Event::Start(Tag::Item) => {
                in_item += 1;
                out.push_str(item.0);
            }
            Event::End(TagEnd::Item) => {
                in_item -= 1;
                out.push_str(item.1);
            }
            Event::End(TagEnd::Paragraph) if in_item == 0 => out.push_str("\n\n"),
            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => out.push('*'),
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => out.push('_'),
            Event::Start(Tag::Link { dest_url, .. }) => {
                out.push_str(&format!("#link(\"{}\")[", typst_string(&dest_url)));
            }
            Event::End(TagEnd::Link) => out.push(']'),
            // The class set code in the body face, and so does this.
            Event::Text(text) | Event::Code(text) => out.push_str(&typst_escape(&text)),
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("\\\n"),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Everything Typst would read as markup rather than as text. Dashes, quotes
/// and `...` are left alone: Typst turns them into the same dashes, curly
/// quotes and ellipses that the TeX ligatures did.
fn typst_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if "\\#$*_[]<>@~`".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
    let typ_only = args.iter().any(|a| a == "--typ");
    let mut variants: Vec<&str> = args.iter().map(String::as_str).filter(|a| !a.starts_with("--")).collect();
    if variants.is_empty() {
        variants.push("default");
    }
    let cfg = Config::load("site.toml");
    let template = Template::load(DIR, "resume.typ");
    fs::create_dir_all(OUT)?;

    for variant in variants {
        let (name, path) = locate(variant);
        let resume = Resume::load(&path)?;
        let job = format!("{}_{name}_resume", slug(resume.doc.get("name")));
        let typ = Path::new(OUT).join(format!("{job}.typ"));
        fs::write(&typ, resume.typst(&template, &cfg.base_url()))?;
        if typ_only {
            println!("wrote {}", typ.display());
            continue;
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

        let publish = cfg.get("resume_pdf");
        if name == "default" && !publish.is_empty() {
            let target = Path::new("static").join(publish);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&pdf, &target)?;
            println!("copied to {}", target.display());
        }
    }
    Ok(())
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
