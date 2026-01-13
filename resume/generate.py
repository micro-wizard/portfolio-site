import argparse
from jinja2 import Environment, FileSystemLoader
import json
import re
from datetime import datetime


def sanitize_input(input: str) -> str:
    input = re.sub("&", "\\&", input)
    return input


def format_link_to_latex(link: str) -> str:
    link = link.replace("_", r"\_")
    return f"{{\\color{{blue}}\\href{{https://{link}}}{{{link}}}}}"


def date_range_to_latex(
    start_year: str | bool,
    start_month: str | bool,
    end_year: str | bool,
    end_month: str | bool,
    no_start_preamble: str = "Graduated",
) -> str:
    if not start_year and end_year:
        return f"{no_start_preamble} \\DatestampYMD{{{end_year}}}{{{end_month}}}"
    if start_year and not end_year:
        return f"\\DatestampYMD{{{start_year}}}{{{start_month}}} --- Present"
    if start_year and end_year:
        return f"\\DatestampYMD{{{start_year}}}{{{start_month}}} --- \\DatestampYMD{{{end_year}}}{{{end_month}}}"
    return ""


def date_range_to_html(
    start_year: str | bool,
    start_month: str | bool,
    end_year: str | bool,
    end_month: str | bool,
    no_start_preamble: str = "Graduated",
) -> str:
    if not start_year and end_year:
        return f"{no_start_preamble} {datetime(int(end_year), int(end_month), 1).strftime('%B %Y')}"
    if start_year and not end_year:
        return f"{datetime(int(start_year), int(start_month), 1).strftime('%B %Y')} - Present"
    if start_year and end_year:
        return f"{datetime(int(start_year), int(start_month), 1).strftime('%B %Y')} - {datetime(int(end_year), int(end_month), 1).strftime('%B %Y')}"
    return ""


def create_latex_section(title: str) -> str:
    return f"\\Section{{{title}}}{{{title}}}{{PDF:{title}}}"


# -------------------------
# Argument Parser
# -------------------------
parser = argparse.ArgumentParser(description="Generate LaTeX + HTML resume")

parser.add_argument(
    "--json",
    default="json/default.json",
    help="Path to JSON resume data file",
)

parser.add_argument(
    "--no-html",
    action="store_true",
    help="Do not generate HTML output",
)

parser.add_argument(
    "--exclude",
    nargs="*",
    default=[],
    help="Sections to exclude: summary work education projects skills",
)

args = parser.parse_args()
excluded = set(args.exclude)


# -------------------------
# Template Environment
# -------------------------
env = Environment(
    loader=FileSystemLoader("."),
    block_start_string="((*",
    block_end_string="*))",
    variable_start_string="(((",
    variable_end_string=")))",
    comment_start_string="((#",
    comment_end_string="#))",
)


# -------------------------
# Load JSON data
# -------------------------
with open(args.json, "r") as f:
    data = json.load(f)


# -------------------------
# Subtitle (always included)
# -------------------------
latex_subtitle = env.get_template("tex_src/subtitle.tex.tpl").render(
    phone_number=data["subtitle"]["phone_number"],
    email=data["subtitle"]["email"],
    location=data["subtitle"]["location"],
    links="\\,\\SubBulletSymbol\\,".join(
        [format_link_to_latex(link) for link in data["subtitle"]["links"]]
    ),
)

html_feature_template = env.get_template("tsx_src/feature_section.tsx.tpl")


# -------------------------
# Summary
# -------------------------
if "summary" in excluded or data["summary"] == "":
    latex_summary = ""
else:
    latex_summary = create_latex_section("Summary") + data["summary"]


# -------------------------
# Work Experience
# -------------------------
jobs = data["jobs"]
if "work" in excluded or len(jobs) == 0:
    latex_work = ""
    html_work = ""
else:
    latex_work_template = env.get_template("tex_src/work.tex.tpl")
    latex_work = create_latex_section("Work Experience")
    latex_work += "\n".join(
        latex_work_template.render(
            company=job["company"],
            url=job["url"],
            location=job["location"],
            dates=date_range_to_latex(
                job["start_year"], job["start_month"], job["end_year"], job["end_month"]
            ),
            job_title=job["job_title"],
            bullet_points="".join(
                [f"\\SubBulletItem\n{line}\n" for line in job["bullet_points"]]
            ),
        )
        for job in jobs
    )

    if not args.no_html:
        html_work_template = env.get_template("tsx_src/job.html.tpl")
        html_work = html_feature_template.render(
            title="Work Experience",
            svg=data["jobs_svg"],
            html="\n".join(
                html_work_template.render(
                    title=job["job_title"],
                    company=job["company"],
                    employment_length=date_range_to_html(
                        job["start_year"],
                        job["start_month"],
                        job["end_year"],
                        job["end_month"],
                    ),
                    bullet_points="".join(
                        [f"<li>{line}</li>" for line in job["bullet_points"]]
                    ),
                )
                for job in jobs
            ),
        )
    else:
        html_work = ""


# -------------------------
# Education
# -------------------------
degrees = data["degrees"]
if "education" in excluded or len(degrees) == 0:
    latex_education = ""
    html_education = ""
else:
    latex_education_template = env.get_template("tex_src/education.tex.tpl")
    latex_education = create_latex_section("Education")
    latex_education += "\n".join(
        latex_education_template.render(
            school=degree["school"],
            url=degree["url"],
            location=degree["location"],
            dates=date_range_to_latex(
                degree["start_year"],
                degree["start_month"],
                degree["end_year"],
                degree["end_month"],
            ),
            degree_name=degree["degree_name"],
            degree_specialization=f" -- \\textit{{{degree['degree_specialization']}}}"
            if degree["degree_specialization"]
            else "",
            bullet_points="".join(
                [f"\\SubBulletItem\n{line}\n" for line in degree["bullet_points"]]
            ),
        )
        for degree in degrees
    )

    if not args.no_html:
        html_education_template = env.get_template("tsx_src/education.html.tpl")
        html_education = html_feature_template.render(
            title="Education",
            svg=data["education_svg"],
            html="\n".join(
                html_education_template.render(
                    degree=degree["degree_name"],
                    school=degree["school"],
                    dates=date_range_to_html(
                        degree["start_year"],
                        degree["start_month"],
                        degree["end_year"],
                        degree["end_month"],
                    ),
                    bullet_points="".join(
                        [f"<li>{line}</li>" for line in degree["bullet_points"]]
                    ),
                )
                for degree in degrees
            ),
        )
    else:
        html_education = ""


# -------------------------
# Projects
# -------------------------
if "projects" in excluded or len(data["projects"]) == 0:
    latex_projects = ""
else:
    latex_projects = create_latex_section("Projects")
    latex_projects_template = env.get_template("tex_src/projects.tex.tpl")
    latex_projects += "\n".join(
        latex_projects_template.render(
            name=p["name"],
            overview=p["overview"],
            link=format_link_to_latex(p["url"]),
            bullet_points="".join(
                [f"\\SubBulletItem\n{line}\n" for line in p["bullet_points"]]
            ),
        )
        for p in data["projects"]
    )


# -------------------------
# Skills
# -------------------------
if "skills" in excluded or len(data["skills"]) == 0:
    latex_skills = ""
else:
    latex_skills = create_latex_section("Skills")
    latex_skills_template = env.get_template("tex_src/skills.tex.tpl")
    latex_skills += "\n".join(
        latex_skills_template.render(
            name=s["name"],
            items=", ".join(s["items"]),
        )
        for s in data["skills"]
    )


# -------------------------
# Write LaTeX resume
# -------------------------
with open("tex_src/nathanSpeltsResume.tex", "w") as out:
    out.write(
        env.get_template("tex_src/nathanSpeltsResume.tex.tpl").render(
            name=data["name"],
            pdf_title=data["pdf_title"],
            website=data["website"],
            subtitle=sanitize_input(latex_subtitle),
            summary=sanitize_input(latex_summary),
            work=sanitize_input(latex_work),
            education=sanitize_input(latex_education),
            projects=sanitize_input(latex_projects),
            skills=sanitize_input(latex_skills),
        )
    )


# -------------------------
# Write HTML (optional)
# -------------------------
if not args.no_html:
    with open("tsx_src/index.tsx", "w") as out:
        out.write(
            env.get_template("tsx_src/index.tsx.tpl").render(
                jobs=html_work,
                education=html_education,
            )
        )
