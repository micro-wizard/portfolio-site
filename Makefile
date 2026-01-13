DATA ?= resume/json/default.json
OUTDIR ?= build

.PHONY: resume
resume:
	pwd
	$(MAKE) -C resume DATA=../$(DATA) OUTDIR=$(OUTDIR)
	open $(OUTDIR)/nathan_spelts_$(basename $(notdir $(DATA)))_resume.pdf -a firefox
