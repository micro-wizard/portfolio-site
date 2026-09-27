---
title: Three Particle Simulators
slug: three_particle_simulators
image: blog/three-particle-simulators/particles.png
summary: How a janky C++ physics project became a JavaScript falling-sand toy, and then a Rust physics engine running in WebGPU compute shaders.
tags: rust, webgpu, graphics, simulation
---

I've now written the same program three times. Each version is a sandbox where
you pick a material, draw it, and watch sand pile up, water spread out and lava
turn to stone. And each one took a different answer to the question at its
core: how much real physics do you need?

1. **[C.A.N.G.-Phys-Sim](https://github.com/Cyrus-Santiago/C.A.N.G.-Phys-Sim)**
   (2022, C++ and OpenGL): real physics, extremely janky.
2. **[Particle-Simulator](https://github.com/micro-wizard/Particle-Simulator)**
   (2022, JavaScript and WebGL): no physics, just cellular automata rules.
   [It's still playable here](/particle-simulator/).
3. **[particles](https://github.com/micro-wizard/particles)** (2026, Rust and
   WebGPU): real physics again, this time in compute shaders.
   [You can play with it here](/particles/).

## Round one: physics, sort of

I wrote about C.A.N.G. [back when I finished it](/blog/cang_physics_simulator/).
It was a four-person semester project at WSU, and I talked my teammates into
building a [powder game](https://dan-ball.jp/en/javagame/dust/) clone in C++
and raw OpenGL. Looking back, the thing I notice is how hard we tried to make
it physically accurate.

![The C.A.N.G. window: an empty play area above a grid of buttons for water, lava, birds, rays and a column of periodic table elements](/blog/three-particle-simulators/cang-menu.png)

*C.A.N.G. still runs. We had big plans: everything from hydrogen to uranium
is just a colour, and only water, ice, fire, steam, lava, stone, birds and
lightning got their own behaviour.*

Every entity had a `Physics` component with a mass. Gravity was a
`#define GRAVITY 9.17`. We wrote a whole `PhysCalc` class out of our Physics 1
notes: elastic and inelastic collisions in one and two dimensions, the
coefficient of restitution, conservation of momentum checked with an `assert`.

```cpp
//Kinetic Energy baby! Physics 1! Determines whether collision is elastic or not
bool calcKineticEnergy(float m1, float m2, float u1, float u2);
```

Then the simulation loop mostly ignored it. `PhysCalc` is only called from
`test.cpp`. What ran every frame looked like this:

```cpp
float gravity = reg.get<Physics>(entt).mass * GRAVITY;

auto enttBelow = entityExists(reg, entt, enttR, DOWN);
if (!reg.valid(enttBelow)) {
    moveY(reg, entt, dt, 2, gravity);
}
```

That's a speed proportional to mass, so heavy things fall faster, which is not
how gravity works. Nothing accelerates either, you're falling at full speed or
standing still. Each entity was a rectangle of any size, and to find
collisions it stamped its ID into every pixel it covered in a big 2D grid.
Moving meant erasing yourself from the grid, sliding over, scanning the new
area for overlaps and stamping yourself back in. The borders registered an
extra 10 pixel buffer so fast things wouldn't tunnel through them.

![A cartoon explosion throws red triangles and fire particles outward over two orange blocks](/blog/three-particle-simulators/cang-explosion.jpg)

*The BOOM tool, from our [demo video](https://www.youtube.com/watch?v=1UXlkcL6s_0).
Explosions were one of the few places velocities actually drove motion.*

Water chose left or right at random, and my favourite bug is in how it did
that:

```cpp
srand(time(0) * (uint)entt);
int direction = rand() % 19;
```

`time(0)` counts seconds. Reseeding with it on every call means each water
particle picks the same "random" direction for a full second, then switches.

My teammate Cyrus remembers a worse one:

> Tbf some of the bugs were cursed, like when we tried making the frame an
> object and it was affected by gravity, which was just a constant shift
> downwards lol

None of this is the fault of the physics. It's what happens when you know the
equations but not how to put them on a computer: how to integrate them over
time, and how to find what's touching what without checking every pair. The
project worked, and I'm still proud of it, but it wasn't accurate physics.

## Round two: forget physics

The day after I published that post, I started over in the browser. The first
commit on Particle-Simulator is May 3rd, 2022. This time I copied what falling
sand games actually do: throw out physics and use a cellular automaton.

The world is a grid of cells. Each cell holds one particle or nothing, and every
frame each particle follows a short rule based on its neighbours:

- **Sand** moves down one cell if it can.
- **Water** moves down, and one time in five drifts one cell left or right.
- **Steam** and **fire** do the same, but upwards.
- If the particle below you is less dense, you **swap** places. That one rule
  is how sand sinks through water.

```js
case 'Water':
    rand = Math.floor(Math.random() * 5);
    if (rand === 4) pdx = window.particleSize;
    else if (rand === 0) pdx = -1 * window.particleSize;
    else pdx = 0;
    pdy = window.particleSize;
    break;
```

Reactions are rules too. Water touching lava becomes steam, and the lava turns
to stone. Steam that reaches the ceiling turns back into water. A seed landing
on sand grows a few cells of grass. Flammable things next to fire have a one in
five chance of catching each frame.

The grid itself was a JavaScript `Map` keyed by `x * 1000 + y`, and a `stall`
set tracked which particles had already moved that frame so nothing got
updated twice as it moved down the grid.

It's so much less code than C.A.N.G. and it behaves so much better. Cellular
automata are the right tool for this kind of thing: every rule is local,
nothing can overlap because a cell holds one thing, and "physics" is whatever
rules you write down.

The catch is that the rules are all there is. Nothing has momentum, so sand
doesn't bounce or scatter, it just steps down one cell at a time. Water spreads
because of a dice roll, not pressure. Every new behaviour is another special
case in a big `switch` statement, and it all runs one particle at a time on the
CPU.

![Blocky pixel sand stacked in tall straight towers with grass on top, a flat pool of water, and lava next to grey stone with steam and water drops drifting up](/blog/three-particle-simulators/particle-simulator.png)

*Particle-Simulator. Sand only ever moves straight down, so instead of forming
slopes it stacks into towers. On the right, water that touched lava rises as
steam and turns back into water at the ceiling.*

## Round three: physics, properly

This month I rewrote it again in Rust, running on the GPU through
[wgpu](https://wgpu.rs/) and WebGPU. This time the particles aren't tied to
cells. Each one has a position, a velocity, a radius, a mass and a temperature,
and it moves because forces act on it.

Solid grains use the discrete element method. When two grains overlap they
push apart with a spring, a damper absorbs some of the energy, and friction
resists them sliding past each other up to a Coulomb limit:

```wgsl
let f_n = max(m.k_n * overlap - gamma_n * approach, 0.0);
...
let limit = m.mu * f_n;
```

Each material sets its own stiffness, damping and friction, so gravel with high
friction piles up into steep heaps and powder slumps. Grains remember how far
they've been sheared against each neighbour from one step to the next, which is
what lets a pile stand up at all instead of slowly melting flat.

Fluids use smoothed particle hydrodynamics. Each particle measures how crowded
its neighbourhood is, crowding becomes pressure, and pressure pushes particles
apart. There's viscosity too, which is how lava ends up thicker than water. Heat
conducts between neighbours, and every material has temperatures where it
turns into something else, so lava cools into obsidian and water boils into
steam because of their temperatures, not because a rule says "water next to
lava". Gravity is even correct now:

```wgsl
fn weight_of(me: Grain) -> vec2<f32> {
    return vec2<f32>(0.0, params.gravity * (me.mass - params.ambient_density * me.volume));
}
```

That's weight minus buoyancy, which is why steam rises. It turned out to be
the fix for the thing I got wrong in 2022.

![A sloped sand bank streaked with gravel, a lava pool crusted with dark obsidian where it meets water, and clouds of steam and condensed water drifting along the ceiling](/blog/three-particle-simulators/particles.png)

*particles, a few seconds after pouring lava into water. The sand settles into a
slope with gravel mixed through it, the lava's surface has cooled into
obsidian, and the boiled-off steam is condensing into water again up top.*

### Doing it on the GPU

The reason this works now and didn't then is the GPU. Up to 70,000 particles
each need to find their neighbours, add up forces and move, 500 times per
simulated second. Every particle can do that at the same time, so it maps
really well onto compute shaders.

Each step is four compute passes:

1. **Scan**: every particle has already counted itself into a cell of a
   uniform grid. A prefix sum over the counts gives each cell a starting offset.
2. **Scatter**: each particle copies itself into its cell's slot, so particles
   in the same cell end up next to each other in memory. It's a counting sort,
   and it replaces C.A.N.G.'s stamp-yourself-into-every-pixel grid.
3. **Plants**: seeds grow into stems that bend like springs.
4. **Solve**: each particle reads the nearby cells, adds up contact, pressure,
   viscous and heat terms, and integrates its velocity and position.

The grid is only for finding neighbours. Particles live in continuous space and
the cells are just big enough that anything touching you is in one of the
surrounding cells.

The timestep is fixed at 1/500 of a second. Springs stiff enough to hold up a
sand pile explode if you take too big a step, so the CPU keeps an accumulator
and runs however many substeps the frame needs, up to a cap. Most of the
GPU-specific trouble came from things being in flight at once. The contact
history is double buffered, so each step reads last step's contacts while
writing its own. New particles get their slots from a free list that's handled
with atomics.

## What I'd tell myself in 2022

C.A.N.G. had the right idea and the wrong tools. We knew some of the physics,
but not how to integrate it or how to find neighbours quickly, and without
those the equations had nowhere to go. Particle-Simulator was the honest
response: give up on physics and write rules instead. It was more fun than
C.A.N.G. and a lot easier to build.

particles comes back to the first idea with the parts that were missing: a
fixed timestep, spatial hashing, forces from well-understood models, and a GPU
to do all of it for every particle at once. I spend less time writing special
cases and more time tuning materials, and a lot of behaviour I never wrote a
rule for, like steam drifting on the wind, falls out of the same few forces.

[Go draw some lava](/particles/)!
