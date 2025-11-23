from jinja2 import Environment, FileSystemLoader
import json


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


def create_latex_section(title: str) -> str:
    return f"\\Section{{{title}}}{{{title}}}{{PDF:{title}}}"


env = Environment(
    loader=FileSystemLoader("."),
    block_start_string="((*",
    block_end_string="*))",
    variable_start_string="(((",
    variable_end_string=")))",
    comment_start_string="((#",
    comment_end_string="#))",
)

with open("tex_src/default.json", "r") as f:
    data = json.load(f)


subtitle = env.get_template("tex_src/subtitle.tex.tpl").render(
    phone_number=data["subtitle"]["phone_number"],
    email=data["subtitle"]["email"],
    location=data["subtitle"]["location"],
    links="\\,\\SubBulletSymbol\\,".join(
        [format_link_to_latex(link) for link in data["subtitle"]["links"]]
    ),
)

if data["summary"] != "":
    summary = create_latex_section("Summary")
    summary += data["summary"]
else:
    summary = ""

if len(data["jobs"]) > 0:
    work = create_latex_section("Work Experience")
else:
    work = ""
work_template = env.get_template("tex_src/work.tex.tpl")
work += "\n".join(
    work_template.render(
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
    for job in data["jobs"]
)

if len(data["degrees"]) > 0:
    education = create_latex_section("Education")
else:
    education = ""
education_template = env.get_template("tex_src/education.tex.tpl")
education += "\n".join(
    education_template.render(
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
    for degree in data["degrees"]
)

if len(data["projects"]) > 0:
    projects = create_latex_section("Projects")
else:
    projects = ""
projects_template = env.get_template("tex_src/projects.tex.tpl")
projects += "\n".join(
    projects_template.render(
        name=project["name"],
        overview=project["overview"],
        link=format_link_to_latex(project["url"]),
        bullet_points="".join(
            [f"\\SubBulletItem\n{line}\n" for line in project["bullet_points"]]
        ),
    )
    for project in data["projects"]
)

if len(data["skills"]) > 0:
    skills = create_latex_section("Skills")
else:
    skills = ""
skills_template = env.get_template("tex_src/skills.tex.tpl")
skills += "\n".join(
    skills_template.render(
        name=skill["name"],
        items=", ".join([line for line in skill["items"]]),
    )
    for skill in data["skills"]
)


with open("tex_src/nathanSpeltsResume.tex", "w") as out:
    out.writelines(
        env.get_template("tex_src/nathanSpeltsResume.tex.tpl").render(
            name=data["name"],
            pdf_title=data["pdf_title"],
            website=data["website"],
            subtitle=subtitle,
            summary=summary,
            work=work,
            education=education,
            projects=projects,
            skills=skills,
        )
    )
