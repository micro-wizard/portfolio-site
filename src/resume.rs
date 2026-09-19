//! The resume: one markdown file rendered two ways — HTML for the front page
//! and LaTeX (resume/simpleresumecv.cls) for the PDF.
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
//! `name [at] domain [dot] com`. The site never prints it as text; the link
//! is assembled by script when someone reaches for it. `phone:` only ever
//! appears in the PDF. A file marked `public: true` (the front page) prints
//! no phone and spells the email out as `[at]`/`[dot]` in its PDF too, since
//! that PDF is linked from the site.

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

    /// The contact links under the name on the front page.
    pub fn web_links(&self) -> String {
        let mut out = Vec::new();
        if !self.email.is_empty() {
            out.push(email_link(&self.email, "Email"));
        }
        for link in self.links.iter().filter(|l| l.only.web()) {
            out.push(format!("<a href=\"{}\">{}</a>", escape(&link.href), escape(&link.label)));
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

    /// A complete .tex document for resume/resume.tex.
    pub fn latex(&self, template: &Template, website: &str) -> String {
        let mut body = String::new();
        for section in self.sections.iter().filter(|s| s.only.pdf()) {
            let entries: Vec<&Entry> = section.entries.iter().filter(|e| e.only.pdf()).collect();
            if section.intro.trim().is_empty() && entries.is_empty() {
                continue;
            }
            if !section.title.is_empty() {
                let title = latex(&section.title, "");
                body.push_str(&format!("\\Section{{{title}}}{{{title}}}{{PDF:{title}}}\n"));
            }
            let intro = latex(&section.intro, "\\Entry\n");
            if !intro.is_empty() {
                body.push_str(&intro);
                body.push_str("\n\n");
            }
            for entry in entries {
                body.push_str(&latex_entry(entry));
            }
            body.push('\n');
        }

        let doc = &self.doc;
        template.render(&[
            ("name", &latex(doc.get("name"), "")),
            ("pdf_title", &latex(doc.get("pdf_title"), "")),
            ("website", &latex_url(website)),
            ("subtitle", &self.latex_subtitle(website)),
            ("body", &body),
        ])
    }

    /// Phone · email · location, then the remaining links on their own line.
    fn latex_subtitle(&self, website: &str) -> String {
        const SEP: &str = "\n\\,\\SubBulletSymbol\\,\n";
        let pdf_links: Vec<&Link> = self.links.iter().filter(|l| l.only.pdf()).collect();
        let mut first = Vec::new();
        let phone = self.doc.get("phone");
        if !phone.is_empty() && !self.public {
            first.push(latex(phone, ""));
        }
        if self.public && !self.email.is_empty() {
            first.push(latex(&mask_email(&self.email), ""));
        } else if !self.email.is_empty() {
            let email = &self.email;
            first.push(format!("\\href{{mailto:{}}}\n{{{}}}", latex_url(email), latex(email, "")));
        }
        let location = self.doc.get("location");
        if !location.is_empty() {
            first.push(format!("{{{}}}", latex(location, "")));
        }
        let second: Vec<String> = pdf_links
            .iter()
            .map(|l| {
                let href = if l.href.starts_with('/') {
                    format!("{website}{}", l.href)
                } else {
                    l.href.clone()
                };
                let shown = href
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .trim_end_matches('/');
                format!("{{\\color{{blue}}\\href{{{}}}{{{}}}}}", latex_url(&href), latex(shown, ""))
            })
            .collect();
        let mut out = String::from("\\begin{SubTitle}\n");
        out.push_str(&first.join(SEP));
        if !second.is_empty() {
            out.push_str("\n\\par\n");
            out.push_str(&second.join("\\,\\SubBulletSymbol\\,"));
        }
        out.push_str("\n\\end{SubTitle}");
        out
    }
}

/// `you [at] example [dot] com` -> `you@example.com`; plain addresses
/// pass through.
fn unmask_email(src: &str) -> String {
    src.trim().replace(" [at] ", "@").replace(" [dot] ", ".")
}

fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((user, domain)) => format!("{user} [at] {}", domain.replace('.', " [dot] ")),
        None => email.to_string(),
    }
}

/// A mailto link with no address in the markup: `data-email` holds it
/// reversed and rot13'd, and templates/base.html restores it on hover, focus
/// or touch.
pub fn email_link(email: &str, label: &str) -> String {
    let hidden: String = email
        .chars()
        .rev()
        .map(|c| match c {
            'a'..='m' | 'A'..='M' => (c as u8 + 13) as char,
            'n'..='z' | 'N'..='Z' => (c as u8 - 13) as char,
            _ => c,
        })
        .collect();
    format!("<a href=\"#\" data-email=\"{}\">{}</a>", escape(&hidden), escape(label))
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

fn latex_entry(entry: &Entry) -> String {
    let title = latex(&entry.title, "");
    let mut out = String::from("\\Entry\n");
    let (lead, dates) = match entry.meta.as_slice() {
        [] => (None, None),
        [only] => (Some(only), None),
        [role, rest @ ..] => (Some(role), Some(rest)),
    };
    match dates {
        // A job or degree: the heading links the organisation, the dates sit
        // on the right and the role goes on the bullet line below.
        Some(rest) => {
            let (places, dates) = rest.split_at(rest.len() - 1);
            if entry.url.is_empty() {
                out.push_str(&format!("\\textbf{{{title}}}"));
            } else {
                out.push_str(&format!("\\href{{{}}}\n{{\\textbf{{{title}}}}}", latex_url(&entry.url)));
            }
            for place in places {
                out.push_str(&format!(",\n{}", latex(place, "")));
            }
            out.push_str(&format!("\n\\hfill\n{}\n", latex_dates(&dates[0])));
        }
        // A project: its link is spelled out on the right.
        None => {
            out.push_str(&format!("\\textbf{{{title}}}\n\\hfill\n"));
            if !entry.url.is_empty() {
                let shown = entry.url.trim_start_matches("https://").trim_start_matches("http://");
                out.push_str(&format!(
                    "{{\\color{{blue}}\\href{{{}}}{{{}}}}}\n",
                    latex_url(&entry.url),
                    latex(shown.trim_end_matches('/'), "")
                ));
            }
        }
    }
    if let Some(lead) = lead {
        out.push_str(&format!("\n\\Gap\n\\BulletItem\n{}\n", latex(lead, "")));
    }
    let body = latex(&entry.body, "\\SubBulletItem\n");
    if !body.is_empty() {
        out.push_str(&format!("\n\\Gap\n\\begin{{Detail}}\n{body}\n\\end{{Detail}}\n"));
    }
    out.push_str("\\Gap\n");
    out
}

/// `Aug 2023 – Present` -> `\mbox{Aug 2023} --- \mbox{Present}`, matching the
/// class's own date stamps.
fn latex_dates(src: &str) -> String {
    latex(src, "")
        .split(['–', '—'])
        .map(|part| format!("\\mbox{{{}}}", part.trim()))
        .collect::<Vec<_>>()
        .join(" --- ")
}

/// Markdown to LaTeX. List items are introduced with `item`; nested lists
/// flatten, and inline HTML is dropped.
fn latex(src: &str, item: &str) -> String {
    let mut out = String::new();
    let mut in_item = 0;
    for event in Parser::new_ext(src, crate::markdown_options()) {
        match event {
            Event::Start(Tag::Item) => {
                in_item += 1;
                out.push_str(item);
            }
            Event::End(TagEnd::Item) => {
                in_item -= 1;
                out.push('\n');
            }
            Event::End(TagEnd::Paragraph) if in_item == 0 => out.push_str("\n\n"),
            Event::Start(Tag::Strong) => out.push_str("\\textbf{"),
            Event::Start(Tag::Emphasis) => out.push_str("\\textit{"),
            Event::Start(Tag::Link { dest_url, .. }) => {
                out.push_str(&format!("\\href{{{}}}{{", latex_url(&dest_url)));
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Link) => out.push('}'),
            Event::Text(text) => out.push_str(&latex_escape(&text)),
            Event::Code(code) => out.push_str(&format!("\\texttt{{{}}}", latex_escape(&code))),
            Event::SoftBreak => out.push('\n'),
            Event::HardBreak => out.push_str("\\\\\n"),
            _ => {}
        }
    }
    out.trim().to_string()
}

fn latex_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '&' | '%' | '$' | '#' | '_' | '{' | '}' => {
                out.push('\\');
                out.push(c);
            }
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            _ => out.push(c),
        }
    }
    out
}

/// hyperref takes URLs nearly verbatim; only these would break the argument.
fn latex_url(url: &str) -> String {
    url.replace('\\', "/").replace('%', "\\%").replace('#', "\\#").replace(['{', '}'], "")
}

/// `site resume [--tex] [variant ...]`: write the .tex for each variant and,
/// unless `--tex` is given, typeset it with latexmk. The default variant is
/// content/index.md, the same file as the front page, and its PDF is copied
/// into static/ for the site to serve.
pub fn command(args: &[String]) -> io::Result<()> {
    let tex_only = args.iter().any(|a| a == "--tex");
    let mut variants: Vec<&str> = args.iter().map(String::as_str).filter(|a| !a.starts_with("--")).collect();
    if variants.is_empty() {
        variants.push("default");
    }
    let cfg = Config::load("site.toml");
    let template = Template::load(DIR, "resume.tex");
    fs::create_dir_all(OUT)?;

    for variant in variants {
        let (name, path) = locate(variant);
        let resume = Resume::load(&path)?;
        let job = format!("{}_{name}_resume", slug(resume.doc.get("name")));
        let tex = Path::new(OUT).join(format!("{job}.tex"));
        fs::write(&tex, resume.latex(&template, &cfg.base_url()))?;
        if tex_only {
            println!("wrote {}", tex.display());
            continue;
        }

        // The class loads its fonts from ./Fonts, so typeset from resume/.
        let status = Command::new("latexmk")
            .current_dir(DIR)
            .args(["-xelatex", "-interaction=nonstopmode", "-halt-on-error", "-file-line-error"])
            .arg("-outdir=build")
            .arg(format!("build/{job}.tex"))
            .status()
            .map_err(|e| io::Error::new(e.kind(), format!("cannot run latexmk: {e}")))?;
        if !status.success() {
            return Err(io::Error::other(format!("latexmk failed on {}", tex.display())));
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
