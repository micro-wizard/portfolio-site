DATA ?= json/default.json
OUTDIR ?= ../build

UV := uv

.PHONY: run
run:
	pwd
	$(UV) run generate.py --json $(DATA)
	$(MAKE) -C tex_src OUTDIR=../../$(OUTDIR) FILENAME=nathan_spelts_$(basename $(notdir ../$(DATA)))_resume
