from jinja2 import Environment, FileSystemLoader, Template
import json

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
        start=job["start"],
        end=job["end"],
        job_title=job["job_title"],
        bullet_points="".join(
            [f"\\SubBulletItem\n{line}\n" for line in job["bullet_points"]]
        ),
    )

education = env.get_template("tex_src/education.tex.tpl").render()

with open("tex_src/nathanSpeltsResume.tex", "w") as out:
    out.writelines(
        env.get_template("tex_src/nathanSpeltsResume.tex.tpl").render(
            work=work, education=education, subtitle=subtitle
        )
    )
