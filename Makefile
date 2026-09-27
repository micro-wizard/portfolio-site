VARIANT ?= default

.PHONY: serve build particles resume monogram

serve: particles
	cargo run -- serve

build: particles
	cargo run --release -- build

# The simulator on /particles/, copied into dist/ by the site build.
particles:
	cd include/particles && trunk build --release --public-url ./

# The 3D front-page monogram, re-exported after editing the Blender model.
monogram:
	blender -b 3d_mongram.blend --python scripts/export_monogram.py

# make resume VARIANT=garmin  ->  resume/variants/garmin.md
resume:
	cargo run -- resume $(VARIANT)
	open -a firefox resume/build/nathan_spelts_$(VARIANT)_resume.pdf
