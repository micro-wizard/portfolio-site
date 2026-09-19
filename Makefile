VARIANT ?= default

.PHONY: serve build resume

serve:
	cargo run -- serve

build:
	cargo run --release -- build

# make resume VARIANT=garmin  ->  resume/variants/garmin.md
resume:
	cargo run -- resume $(VARIANT)
	open -a firefox resume/build/nathan_spelts_$(VARIANT)_resume.pdf
