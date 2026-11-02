// Template for `site cover-letter`: the letter (resume/letters/*.md) under
// the name and contact details of the resume of the same variant. The
// layout is `letter` in lib.typ.
//
// The filled-in copy is written to build/, so the import is spelled from the
// typst root (resume/) rather than relative to this file.

#import "/lib.typ": *

#show: letter.with(author: "{{ author }}", title: "{{ title }}")

#name[{{ name }}]

{{ subtitle }}

#dateline

{{ body }}
