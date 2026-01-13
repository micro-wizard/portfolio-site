
OUTDIR ?= ../../build
FILENAME ?= nathan_spelts_resume

.PHONY: run
run:
	pwd
	mkdir -p $(OUTDIR)
	latexmk -pdf -xelatex \
		-interaction=nonstopmode \
		-file-line-error \
		-halt-on-error \
		-outdir=$(OUTDIR) \
		-jobname=$(FILENAME) \
		nathanSpeltsResume.tex
