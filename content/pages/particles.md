---
title: Particles
description: A falling-sand physics sandbox written in JavaScript and WebGL.
---

A falling-sand sandbox I wrote in JavaScript and WebGL. Pick a material and
draw on the canvas. Sand piles up, water spreads out, lava sets things on fire
and seeds grow.

<div class="sim-controls">
  <button type="button" id="sand-select">Sand <kbd>Q</kbd></button>
  <button type="button" id="water-select">Water <kbd>W</kbd></button>
  <button type="button" id="lava-select">Lava <kbd>E</kbd></button>
  <button type="button" id="seed-select">Seed <kbd>R</kbd></button>
  <button type="button" id="steam-select">Steam <kbd>A</kbd></button>
  <button type="button" id="stone-select">Stone <kbd>S</kbd></button>
  <button type="button" id="border-select">Wall <kbd>D</kbd></button>
  <button type="button" id="fire-select">Fire <kbd>F</kbd></button>
  <button type="button" id="eraser">Erase <kbd>X</kbd></button>
  <button type="button" id="brush-decrease" aria-label="Smaller brush">Brush − <kbd>1</kbd></button>
  <button type="button" id="brush-increase" aria-label="Larger brush">Brush + <kbd>2</kbd></button>
</div>
<div id="div1" class="sim-stage">
  <canvas id="glcanvas" width="600" height="450"></canvas>
  <noscript><p>The simulator needs JavaScript and WebGL.</p></noscript>
</div>
<dl class="sim-stats">
  <div><dt>Material</dt><dd id="SelectionDisplay3">–</dd></div>
  <div><dt>Brush</dt><dd id="SelectionDisplay4">–</dd></div>
  <div><dt>Particles</dt><dd id="SelectionDisplay1">–</dd></div>
  <div><dt>FPS</dt><dd id="SelectionDisplay2">–</dd></div>
  <div><dt>X</dt><dd id="SelectionDisplay5">–</dd></div>
  <div><dt>Y</dt><dd id="SelectionDisplay6">–</dd></div>
</dl>
<script type="module">
  import { main } from "./sim/main.js";
  main();
</script>

Source: [Particle-Simulator on GitHub](https://github.com/micro-wizard/Particle-Simulator).
