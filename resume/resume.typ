// Template for `site resume`: the double-braced placeholders below are
// filled from the resume markdown (content/index.md or resume/variants/*.md).
// The layout lives in lib.typ; nothing here decides anything.
//
// The filled-in copy is written to build/, so the import is spelled from the
// typst root (resume/) rather than relative to this file.

#import "/lib.typ": *

#show: resume.with(author: "{{ author }}", title: "{{ title }}")

#name[{{ name }}]

{{ subtitle }}

{{ body }}
