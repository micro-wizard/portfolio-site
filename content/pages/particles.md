---
title: Particles
description: A GPU particle physics sandbox written in Rust and WebGPU.
---

A particle physics sandbox I wrote in Rust, running on the GPU through wgpu
and WebGPU. Particles aren't tied to grid cells: sand, gravel, powder, water,
ice, steam, lava and obsidian each have their own physics, and heat moves
between them, so lava cools into obsidian and water boils into steam. Pick a
material or a heat brush in the panel and draw; <kbd>F1</kbd> hides the panel.

<iframe class="sim-stage" src="app/" title="Particle simulator" allow="fullscreen"></iframe>

It needs a browser with WebGPU. The frame shows the reason if it can't start.

Source: [particles on GitHub](https://github.com/micro-wizard/particles).
This replaced an older falling-sand sandbox written in JavaScript and WebGL,
[Particle-Simulator](https://github.com/micro-wizard/Particle-Simulator).
