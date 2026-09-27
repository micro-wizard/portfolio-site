const stage = document.querySelector(".mark-stage");
const canvas = stage?.querySelector(".mark-3d");
const svg = stage?.querySelector(".mark-lg");

const MAX_YAW = 0.5, MAX_PITCH = 0.45;
const SCROLL_TURN = 2 * Math.PI;
const FOV = Math.PI / 6;
const COLOURS = { "ns-a": "--fg", "ns-b": "--accent" };

const VERTEX = `#version 300 es
in vec3 position;
uniform vec3 center;
uniform float scale;
uniform vec2 turn; // yaw, pitch
uniform float distance;
uniform mat4 projection;
out vec3 modelPos;
out vec3 viewPos;
void main() {
  vec3 p = (position - center) * scale;
  modelPos = p;
  float cy = cos(turn.x), sy = sin(turn.x), cp = cos(turn.y), sp = sin(turn.y);
  p = vec3(p.x * cy + p.z * sy, p.y, p.z * cy - p.x * sy);
  p = vec3(p.x, p.y * cp - p.z * sp, p.y * sp + p.z * cp);
  p.z -= distance;
  viewPos = p;
  gl_Position = projection * vec4(p, 1.0);
}`;

const FRAGMENT = `#version 300 es
precision highp float;
in vec3 modelPos;
in vec3 viewPos;
uniform vec3 colour;
uniform vec3 background;
out vec4 outColour;
void main() {
  vec3 facing = normalize(cross(dFdx(modelPos), dFdy(modelPos)));
  vec3 n = normalize(cross(dFdx(viewPos), dFdy(viewPos)));
  float light = max(dot(n, normalize(vec3(-0.4, 0.6, 0.7))), 0.0);
  vec3 wall = mix(colour, background, 0.25 + 0.2 * (1.0 - light));
  outColour = vec4(abs(facing.z) > 0.999 ? colour : wall, 1.0);
}`;

async function loadGlb(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  const buf = await res.arrayBuffer();
  const dv = new DataView(buf);
  if (dv.getUint32(0, true) !== 0x46546c67) throw new Error(`${url}: not a .glb`);
  let json, bin = 0;
  for (let off = 12; off < buf.byteLength; ) {
    const len = dv.getUint32(off, true), type = dv.getUint32(off + 4, true);
    if (type === 0x4e4f534a) json = JSON.parse(new TextDecoder().decode(new Uint8Array(buf, off + 8, len)));
    if (type === 0x004e4942) bin = off + 8;
    off += 8 + len;
  }
  const read = (i, Type, size) => {
    const a = json.accessors[i], view = json.bufferViews[a.bufferView];
    return new Type(buf, bin + (view.byteOffset || 0) + (a.byteOffset || 0), a.count * size);
  };
  const INDEX = { 5121: Uint8Array, 5123: Uint16Array, 5125: Uint32Array };
  return json.nodes.filter((n) => n.mesh !== undefined).flatMap((node) =>
    json.meshes[node.mesh].primitives.map((p) => ({
      name: node.name,
      bounds: json.accessors[p.attributes.POSITION],
      positions: read(p.attributes.POSITION, Float32Array, 3),
      indices: read(p.indices, INDEX[json.accessors[p.indices].componentType], 1),
      indexType: json.accessors[p.indices].componentType,
    })));
}

function compile(gl) {
  const program = gl.createProgram();
  for (const [type, src] of [[gl.VERTEX_SHADER, VERTEX], [gl.FRAGMENT_SHADER, FRAGMENT]]) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(shader));
    gl.attachShader(program, shader);
  }
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
  return program;
}

function cssColour(name) {
  const hex = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
}

async function start(mouse) {
  const gl = canvas.getContext("webgl2", { antialias: true });
  if (!gl) return;
  const parts = await loadGlb(canvas.dataset.model);
  const program = compile(gl);
  gl.useProgram(program);
  const uniform = (name) => gl.getUniformLocation(program, name);
  const attrib = gl.getAttribLocation(program, "position");

  for (const part of parts) {
    part.vao = gl.createVertexArray();
    gl.bindVertexArray(part.vao);
    gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ARRAY_BUFFER, part.positions, gl.STATIC_DRAW);
    gl.enableVertexAttribArray(attrib);
    gl.vertexAttribPointer(attrib, 3, gl.FLOAT, false, 0, 0);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, gl.createBuffer());
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, part.indices, gl.STATIC_DRAW);
  }

  const min = [0, 1, 2].map((i) => Math.min(...parts.map((p) => p.bounds.min[i])));
  const max = [0, 1, 2].map((i) => Math.max(...parts.map((p) => p.bounds.max[i])));
  gl.uniform3fv(uniform("center"), min.map((lo, i) => (lo + max[i]) / 2));
  const scale = 2 / Math.max(...max.map((hi, i) => hi - min[i]));
  gl.uniform1f(uniform("scale"), scale);
  const f = 1 / Math.tan(FOV / 2), near = 1, far = 20;
  const front = ((max[2] - min[2]) / 2) * scale;
  gl.uniform1f(uniform("distance"), front + (canvas.clientWidth / stage.clientWidth) * f);
  gl.uniformMatrix4fv(uniform("projection"), false, [
    f, 0, 0, 0,  0, f, 0, 0,
    0, 0, (far + near) / (near - far), -1,  0, 0, (2 * far * near) / (near - far), 0,
  ]);
  gl.enable(gl.DEPTH_TEST);
  gl.clearColor(0, 0, 0, 0);

  const readColours = () => {
    for (const p of parts) p.colour = cssColour(COLOURS[p.name] || "--fg");
    gl.uniform3fv(uniform("background"), cssColour("--bg"));
  };
  readColours();

  let yaw = 0, pitch = 0, aimYaw = 0, aimPitch = 0;
  let frame = 0, last = 0;

  function draw() {
    const dpr = devicePixelRatio || 1;
    const w = Math.round(canvas.clientWidth * dpr), h = Math.round(canvas.clientHeight * dpr);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    gl.viewport(0, 0, w, h);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.uniform2f(uniform("turn"), yaw, pitch);
    for (const p of parts) {
      gl.uniform3fv(uniform("colour"), p.colour);
      gl.bindVertexArray(p.vao);
      gl.drawElements(gl.TRIANGLES, p.indices.length, p.indexType, 0);
    }
  }

  function tick(now) {
    const k = 1 - Math.exp(-(last ? now - last : 16) / 120);
    last = now;
    yaw += (aimYaw - yaw) * k;
    pitch += (aimPitch - pitch) * k;
    draw();
    const moving = Math.abs(aimYaw - yaw) + Math.abs(aimPitch - pitch) > 1e-4;
    frame = moving ? requestAnimationFrame(tick) : 0;
    if (!moving) last = 0;
  }
  const wake = () => { if (!frame) frame = requestAnimationFrame(tick); };
  const clamp = (v) => Math.max(-1, Math.min(1, v));

  if (mouse) {
    addEventListener("pointermove", (e) => {
      const r = canvas.getBoundingClientRect();
      aimYaw = clamp((e.clientX - r.left - r.width / 2) / (innerWidth / 2)) * MAX_YAW;
      aimPitch = clamp((e.clientY - r.top - r.height / 2) / (innerHeight / 2)) * MAX_PITCH;
      wake();
    }, { passive: true });
    addEventListener("pointerout", (e) => {
      if (e.relatedTarget) return;
      aimYaw = 0;
      aimPitch = 0;
      wake();
    });
  } else {
    const turn = () => {
      const gone = stage.getBoundingClientRect().bottom + scrollY;
      aimYaw = Math.min(scrollY / gone, 1) * SCROLL_TURN;
      wake();
    };
    addEventListener("scroll", turn, { passive: true });
    turn();
  }

  const recolour = () => { readColours(); draw(); };
  new MutationObserver(recolour).observe(document.documentElement, { attributeFilter: ["data-theme"] });
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", recolour);
  addEventListener("resize", draw);
  canvas.addEventListener("webglcontextlost", () => stage.classList.remove("is-3d"));

  draw();
  await Promise.all(svg.getAnimations({ subtree: true }).map((a) => a.finished));
  stage.classList.add("is-3d");
}

const mouse = matchMedia("(hover: hover) and (pointer: fine)").matches;
const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
if (canvas && !calm) start(mouse).catch((e) => console.warn("monogram:", e));
