//! Static site generator: a resume front page plus a markdown blog.
//!
//!   site build          render ./content into ./dist
//!   site serve [port]   build, then serve ./dist with rebuild-on-change
//!   site resume [--typ] [variant ...]
//!                       typeset the resume PDF from the same markdown

mod config;
mod frontmatter;
mod resume;
mod serve;
mod template;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use config::Config;
use frontmatter::Doc;
use pulldown_cmark::{html, Options, Parser};
use template::{escape, Template};

const OUT: &str = "dist";
const CONTENT: &str = "content";
const TEMPLATES: &str = "templates";
const STATIC: &str = "static";
/// The particle-simulator submodule's ES modules, served as-is next to the
/// page that loads them (content/pages/particles.md).
const SIMULATOR: &str = "include/particle-simulator/src";

struct Post {
    slug: String,
    title: String,
    date: String,
    summary: String,
    tags: Vec<String>,
    body: String,
    minutes: usize,
}

impl Post {
    fn url(&self, root: &str) -> String {
        format!("{root}blog/{}/", self.slug)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str).unwrap_or("build") {
        "build" => match build() {
            Ok(n) => println!("built {n} pages -> {OUT}/"),
            Err(e) => fail(&e),
        },
        "serve" => {
            let port: u16 = args.get(1).and_then(|p| p.parse().ok()).unwrap_or(8080);
            if let Err(e) = build() {
                fail(&e);
            }
            serve::run(OUT, port, build);
        }
        "resume" => {
            if let Err(e) = resume::command(&args[1..]) {
                fail(&e);
            }
        }
        other => {
            eprintln!(
                "unknown command: {other}\n\
                 usage: site [build | serve [port] | resume [--typ] [variant ...]]"
            );
            std::process::exit(2);
        }
    }
}

fn fail(e: &io::Error) -> ! {
    eprintln!("error: {e}");
    std::process::exit(1);
}

fn build() -> io::Result<usize> {
    let cfg = Config::load("site.toml");
    let root = cfg.base_path();
    let base_url = cfg.base_url();
    let site_title = cfg.get("title");

    let base = Template::load(TEMPLATES, "base.html");
    let resume_tpl = Template::load(TEMPLATES, "resume.html");
    let post_tpl = Template::load(TEMPLATES, "post.html");
    let list_tpl = Template::load(TEMPLATES, "list.html");
    let page_tpl = Template::load(TEMPLATES, "page.html");

    // Start from a clean tree so deleted content cannot linger in the output.
    if Path::new(OUT).exists() {
        fs::remove_dir_all(OUT)?;
    }
    fs::create_dir_all(OUT)?;
    if Path::new(STATIC).exists() {
        copy_dir(Path::new(STATIC), Path::new(OUT))?;
    }
    if Path::new(SIMULATOR).exists() {
        copy_dir(Path::new(SIMULATOR), &Path::new(OUT).join("particles/sim"))?;
    } else {
        eprintln!("warning: {SIMULATOR} missing; run `git submodule update --init`");
    }

    let mut pages = 0;

    // A page is the rendered body wrapped in the shared shell.
    let emit = |path: &str, title: &str, description: &str, body: &str| -> io::Result<()> {
        let canonical = format!("{base_url}{}", path_to_url(path, &root));
        let page_title = if title.is_empty() || title == site_title {
            site_title.to_string()
        } else {
            format!("{title} · {site_title}")
        };
        let html = base.render(&[
            ("page_title", &escape(&page_title)),
            ("description", &escape(description)),
            ("canonical", &canonical),
            ("root", &root),
            ("site_title", &escape(site_title)),
            ("author", &escape(cfg.get("author"))),
            ("year", &current_year()),
            ("content", body),
        ]);
        write(&Path::new(OUT).join(path), &html)
    };

    // --- front page: the resume ------------------------------------------
    let resume = resume::Resume::load(&Path::new(CONTENT).join("index.md"))?;
    let links = resume.web_links();
    let doc = &resume.doc;
    let resume_body = resume_tpl.render(&[
        ("name", &escape(pick(doc.get("name"), site_title))),
        ("tagline", &escape(pick(doc.get("tagline"), cfg.get("tagline")))),
        ("links", &links),
        ("email", &resume.web_email()),
        ("content", &resume.html()),
    ]);
    emit(
        "index.html",
        "",
        pick(doc.get("description"), cfg.get("description")),
        &resume_body,
    )?;
    pages += 1;

    // --- blog -------------------------------------------------------------
    let mut posts = read_posts()?;
    posts.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.slug.cmp(&b.slug)));

    for post in &posts {
        let tags = render_tags(&post.tags);
        let body = post_tpl.render(&[
            ("title", &escape(&post.title)),
            ("date", &escape(&post.date)),
            ("date_human", &escape(&human_date(&post.date))),
            ("reading_time", &post.minutes.to_string()),
            ("tags", &tags),
            ("root", &root),
            ("content", &markdown(&post.body)),
        ]);
        emit(
            &format!("blog/{}/index.html", post.slug),
            &post.title,
            &post.summary,
            &body,
        )?;
        pages += 1;
    }

    let mut items = String::new();
    for post in &posts {
        items.push_str(&format!(
            "<li class=\"entry\">\n  <a class=\"entry-title\" href=\"{url}\">{title}</a>\n  \
             <time datetime=\"{date}\">{human}</time>\n  <p>{summary}</p>\n</li>\n",
            url = post.url(&root),
            title = escape(&post.title),
            date = escape(&post.date),
            human = escape(&human_date(&post.date)),
            summary = escape(&post.summary),
        ));
    }
    if posts.is_empty() {
        items.push_str("<li class=\"entry\"><p>Nothing published yet.</p></li>");
    }
    let intro = match cfg.get("blog_intro") {
        "" => String::new(),
        text => format!("<p class=\"intro\">{}</p>", escape(text)),
    };
    let list_body = list_tpl.render(&[
        ("title", "Writing"),
        ("intro", &intro),
        ("items", &items),
    ]);
    emit("blog/index.html", "Writing", "Posts and notes.", &list_body)?;
    pages += 1;

    // --- standalone pages -------------------------------------------------
    for (slug, doc) in read_pages()? {
        let body = page_tpl.render(&[
            ("title", &escape(doc.get("title"))),
            ("content", &markdown(&doc.body)),
        ]);
        emit(
            &format!("{slug}/index.html"),
            doc.get("title"),
            doc.get("description"),
            &body,
        )?;
        pages += 1;
    }

    // --- feed, sitemap, 404, Pages plumbing -------------------------------
    write(&Path::new(OUT).join("feed.xml"), &feed(&cfg, &posts, &root))?;
    write(&Path::new(OUT).join("sitemap.xml"), &sitemap(&cfg, &posts, &root))?;
    let not_found = page_tpl.render(&[
        ("title", "Not found"),
        ("content", &format!(
            "<p>That page does not exist. Try the <a href=\"{root}\">front page</a> \
             or the <a href=\"{root}blog/\">writing index</a>.</p>"
        )),
    ]);
    emit("404.html", "Not found", "Page not found.", &not_found)?;
    pages += 1;
    // Tells GitHub Pages to serve the tree as-is instead of running Jekyll.
    write(&Path::new(OUT).join(".nojekyll"), "")?;
    let domain = cfg.get("domain");
    if !domain.is_empty() {
        write(&Path::new(OUT).join("CNAME"), &format!("{domain}\n"))?;
    }

    // + the feed and the sitemap
    Ok(pages + 2)
}

fn read_posts() -> io::Result<Vec<Post>> {
    let dir = Path::new(CONTENT).join("blog");
    let drafts = std::env::var("SITE_DRAFTS").is_ok();
    let mut posts = Vec::new();
    for path in markdown_files(&dir)? {
        let src = fs::read_to_string(&path)?;
        let doc = Doc::parse(&src);
        if doc.flag("draft") && !drafts {
            continue;
        }
        let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let (date_from_name, slug_from_name) = split_dated_stem(&stem);
        let date = pick(doc.get("date"), &date_from_name).to_string();
        if date.is_empty() {
            eprintln!("warning: {} has no date, skipping", path.display());
            continue;
        }
        let words = doc.body.split_whitespace().count();
        posts.push(Post {
            slug: pick(doc.get("slug"), &slug_from_name).to_string(),
            title: pick(doc.get("title"), &slug_from_name).to_string(),
            date,
            summary: doc.get("summary").to_string(),
            tags: doc.list("tags"),
            minutes: (words / 200).max(1),
            body: doc.body,
        });
    }
    Ok(posts)
}

fn read_pages() -> io::Result<Vec<(String, Doc)>> {
    let dir = Path::new(CONTENT).join("pages");
    let mut pages = Vec::new();
    for path in markdown_files(&dir)? {
        let src = fs::read_to_string(&path)?;
        let doc = Doc::parse(&src);
        if doc.flag("draft") {
            continue;
        }
        let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        pages.push((pick(doc.get("slug"), &stem).to_string(), doc));
    }
    Ok(pages)
}

fn markdown_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    files.sort();
    Ok(files)
}

fn markdown(src: &str) -> String {
    let mut out = String::with_capacity(src.len() * 3 / 2);
    html::push_html(&mut out, Parser::new_ext(src, markdown_options()));
    out
}

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    options
}

fn render_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let inner: Vec<String> = tags
        .iter()
        .map(|t| format!("<li>{}</li>", escape(t)))
        .collect();
    format!("<ul class=\"tags\">{}</ul>", inner.join(""))
}

fn feed(cfg: &Config, posts: &[Post], root: &str) -> String {
    let base = cfg.base_url();
    let title = escape(cfg.get("title"));
    let author = escape(cfg.get("author"));
    let updated = posts
        .first()
        .map(|p| rfc3339(&p.date))
        .unwrap_or_else(|| rfc3339("1970-01-01"));
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");
    out.push_str(&format!("  <title>{title}</title>\n"));
    out.push_str(&format!("  <id>{base}{root}</id>\n"));
    out.push_str(&format!("  <link href=\"{base}{root}\"/>\n"));
    out.push_str(&format!(
        "  <link rel=\"self\" type=\"application/atom+xml\" href=\"{base}{root}feed.xml\"/>\n"
    ));
    out.push_str(&format!("  <updated>{updated}</updated>\n"));
    out.push_str(&format!("  <author><name>{author}</name></author>\n"));
    for post in posts {
        let url = format!("{base}{}", post.url(root));
        out.push_str("  <entry>\n");
        out.push_str(&format!("    <title>{}</title>\n", escape(&post.title)));
        out.push_str(&format!("    <id>{url}</id>\n"));
        out.push_str(&format!("    <link href=\"{url}\"/>\n"));
        out.push_str(&format!("    <updated>{}</updated>\n", rfc3339(&post.date)));
        if !post.summary.is_empty() {
            out.push_str(&format!("    <summary>{}</summary>\n", escape(&post.summary)));
        }
        out.push_str(&format!(
            "    <content type=\"html\">{}</content>\n",
            escape(&markdown(&post.body))
        ));
        out.push_str("  </entry>\n");
    }
    out.push_str("</feed>\n");
    out
}

fn sitemap(cfg: &Config, posts: &[Post], root: &str) -> String {
    let base = cfg.base_url();
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for loc in [format!("{base}{root}"), format!("{base}{root}blog/")] {
        out.push_str(&format!("  <url><loc>{loc}</loc></url>\n"));
    }
    for post in posts {
        out.push_str(&format!(
            "  <url><loc>{base}{}</loc><lastmod>{}</lastmod></url>\n",
            post.url(root),
            post.date
        ));
    }
    out.push_str("</urlset>\n");
    out
}

/// `2026-01-14-hello-world` -> (`2026-01-14`, `hello-world`).
fn split_dated_stem(stem: &str) -> (String, String) {
    let bytes = stem.as_bytes();
    let dated = bytes.len() > 11
        && bytes[..10]
            .iter()
            .enumerate()
            .all(|(i, b)| if i == 4 || i == 7 { *b == b'-' } else { b.is_ascii_digit() })
        && bytes[10] == b'-';
    if dated {
        (stem[..10].to_string(), stem[11..].to_string())
    } else {
        (String::new(), stem.to_string())
    }
}

fn human_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() != 3 {
        return iso.to_string();
    }
    let month = parts[1].parse::<usize>().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return iso.to_string();
    }
    let day = parts[2].trim_start_matches('0');
    format!("{} {day}, {}", MONTHS[month - 1], parts[0])
}

fn rfc3339(date: &str) -> String {
    format!("{date}T00:00:00Z")
}

/// Year from the system clock, without a date crate.
fn current_year() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let mut days = secs / 86_400;
    let mut year = 1970;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let len = if leap { 366 } else { 365 };
        if days < len {
            return year.to_string();
        }
        days -= len;
        year += 1;
    }
}

fn path_to_url(path: &str, root: &str) -> String {
    let path = path.strip_suffix("index.html").unwrap_or(path);
    format!("{root}{path}")
}

fn pick<'a>(primary: &'a str, fallback: &'a str) -> &'a str {
    if primary.is_empty() {
        fallback
    } else {
        primary
    }
}

fn write(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
