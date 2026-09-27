---
title: Particle-Simulator
description: My 2022 falling-sand sandbox, written in JavaScript and WebGL.
---

The falling-sand sandbox I wrote in 2022 in JavaScript and WebGL, before
[particles](/particles/) replaced it. Every particle sits in a grid cell and
follows a few rules about its neighbours; there's no physics underneath. Pick a
material with the buttons or the keys in brackets and draw. I wrote about how
it compares to the other two in
[Three Particle Simulators](/blog/three_particle_simulators/).

<iframe class="sim-stage classic" src="app/" title="Particle-Simulator"></iframe>
<script>
// The simulator styles itself from the OS colour scheme unless data-theme on
// its root says otherwise, so copy this page's theme toggle into the frame.
// It sizes its canvas once, from the frame's height when it loads; afterwards
// shrink the frame to what it drew so no empty space is left under it. The
// frame is then as big as its contents, so it never scrolls: without this, a
// scrollbar that later appears on this page narrows the frame below the fixed
// canvas and the frame grows scrollbars of its own.
(function () {
  var frame = document.querySelector(".sim-stage");
  var page = document.documentElement;
  function syncTheme() {
    var root = frame.contentDocument && frame.contentDocument.documentElement;
    if (!root) return;
    if (page.dataset.theme) root.dataset.theme = page.dataset.theme;
    else delete root.dataset.theme;
  }
  frame.addEventListener("load", function () {
    syncTheme();
    var root = frame.contentDocument.documentElement;
    root.style.overflow = "hidden";
    frame.style.aspectRatio = "auto";
    frame.style.height = Math.ceil(root.getBoundingClientRect().height) + "px";
  });
  new MutationObserver(syncTheme).observe(page, { attributeFilter: ["data-theme"] });
})();
</script>

Source: [Particle-Simulator on GitHub](https://github.com/micro-wizard/Particle-Simulator).
