VARIANT ?= default

.PHONY: serve build particles resume

serve: particles
	cargo run -- serve

build: particles
	cargo run --release -- build

# The simulator on /particles/, copied into dist/ by the site build.
particles:
	cd include/particles && trunk build --release --public-url ./

# make resume VARIANT=garmin  ->  resume/variants/garmin.md
resume:
	cargo run -- resume $(VARIANT)
	open -a firefox resume/build/nathan_spelts_$(VARIANT)_resume.pdf
