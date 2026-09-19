---
title: CANG Physics Simulator
slug: cang_physics_simulator
summary: A physics simulator created from scratch. Written in C++ using OpenGL and GLFW.
---

Welcome to my first post on my new blog! I'm just about finished up with my junior year in the computer science sequence at Washington State University, and I wanted to talk about my semester project I just presented.

We were allowed to choose any project we wanted, as long as the scope was large enough. So I convinced some of my friends to create something akin to [powder game](https://dan-ball.jp/en/javagame/dust/). I always loved that game as a kid, and was also always curious how it worked under the hood. I've got a lot of interest in low level graphics rendering as well, mainly due to [this video](https://www.youtube.com/watch?v=4O0_-1NaWnY) where [jdh](https://www.youtube.com/channel/UCUzQJ3JBuQ9w-po4TXRJHiA) creates minecraft using C and OpenGL in 48 hours. So naturally, I convinced everyone to use OpenGL for our physics simulator. It felt like we bit off more than we could chew at times, but ultimately the project turned out really cool and I'm extremely proud of it.

### Time for some technical details for those interested.

My biggest contributions to the project were:
- Creating the infrastructure for drawing things to the screen
- Liquid, Gas, and Solid collisions
- State transitions

#### Drawing infrastructure

This part was extrememly tricky and completely new territory for me. I'd experimented with drawing single triangles to the screen, and rendering some 3d shapes. For example:

<iframe width="450" height="200" src="/videos/openglmeme3.mp4" title="Cubes rotating" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
<a href="openglmeme3.mp4"/>

But I'd never attempted anything where new objects were bring drawn on the fly. Thankfully, on the [site](https://learnopengl.com/) where I've learned everything I know about OpenGL, there's a tutorial on how to create breakout! So I followed the [tutorial](https://learnopengl.com/In-Practice/2D-Game/Breakout) and extracted the information I needed to accomplish what I wanted to. Unfortunately this code I got from the tutorial didn't count towards my line requirement on the project, but I learned a ton so the time wasn't entirely wasted.

#### Liquid, Gas, and Solid collisions

This part of the project I can say with confidence that I thought through it and implemented it all on my own. Unfortunately that means making a lot of mistakes along the way. I'm going to skip all the things I tried that didn't work, and get straight to what did work. I ended up creating a double array of entities, an entity being a number that is unique to a particle drawn on the screen. Each entity stores it's own components that determine how it should behave. I created a liquid component, gas component, gravity component etc.

<iframe width="450" height="200" src="/videos/gravity.mp4" title="Gravity demonstration" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>
<a href="gravity.mp4"/>

*A demonstration of gravity*

Then I wrote rules for what should happen to an entity with a specific component, like the gravity component for example. When a particle has the gravity component, and there's not a valid entity in the space below it, it moves down. But if the particle encounters another particle with the border component, it stops moving down. The grid allowed me to check the immediate surroundings of a particle wherever it was, which kept things from getting bogged down too much. From there I started writing a liquid algorithm.

*A demonstration of water*

I tried many things to get realistic behaving water, and what I found to be the most effective was random movement. Every water particle chooses a random number between 0 and 99, if that number is odd it moves left, if it's even it moves right. The amount the particle moves is dependent on the size of the number chosen. As simple as this is, it took a long time to figure out!

*A demonstration of steam*

Steam was fairly simple to figure out, it chooses a random number similarly to water and slowly moves to the left or right as it ascends. Once it reaches a solid object, there's a probability that it will condense back into water every few seconds.

#### State Transitions

This part was my favorite because it was so simple. Once I had collisions and drawing particles all set up, turning one entity into another was as simple as taking away one component and adding another. There's water touching lava? Turn the lava into stone and turn the water into steam! The possibilities for things that could be added are endless. Of course, by the time I got into the stage where I was just adding fun things, I ran out of time unfortunately. But I managed to add water, ice, steam, lava, fire, and stone. I also added birds as my last contribution to the project. I was able to implement them via Boid's algorithm, courtesy of [Ben Eater](https://eater.net/boids).

*A demonstration of birds*

I had a lot of fun with this project and learned a lot about working with others. I feel like I didn't get too technical with my explanations. If you think it was too technical or not technical enough, go ahead and send me an <a href="#" data-email="zbp.fgyrcfanugna@rgna">email</a> and tell me about it! Even if you don't have a specific comment, I'd love to hear that someone actually read this and enjoyed it! Thanks for your time. :)