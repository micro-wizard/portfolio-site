from jinja2 import Environment, FileSystemLoader, Template
import json


def date_range_to_string(
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
    phone_number="(425) 408 - 2655",
    email="nathan@spelts.net",
    location="Vancouver, WA",
    website="nathanspelts.com",
    github_url="https://github.com/micro-wizard",
    github_pretty_url="github.com/micro-wizard",
    linkedin_url="https://www.linkedin.com/in/nathanspelts/",
    linkedin_pretty_url="linkedin.com/in/nathanspelts",
)

work = ""
work_template = env.get_template("tex_src/work.tex.tpl")
for job in data["jobs"]:
    work += work_template.render(
        company=job["company"],
        url=job["url"],
        location=job["location"],
        dates=date_range_to_string(
            job["start_year"], job["start_month"], job["end_year"], job["end_month"]
        ),
        job_title=job["job_title"],
        bullet_points="".join(
            [f"\\SubBulletItem\n{line}\n" for line in job["bullet_points"]]
        ),
    )

education = ""
education_template = env.get_template("tex_src/education.tex.tpl")
for degree in data["degrees"]:
    education += education_template.render(
        school=degree["school"],
        url=degree["url"],
        location=degree["location"],
        dates=date_range_to_string(
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

with open("tex_src/nathanSpeltsResume.tex", "w") as out:
    out.writelines(
        env.get_template("tex_src/nathanSpeltsResume.tex.tpl").render(
            work=work, education=education, subtitle=subtitle
        )
    )
