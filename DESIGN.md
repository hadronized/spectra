# Roadmap & features

The current document is for development only. It contains various information about things to do and
planned features.

## Features

  - Writing demoscene applications should be easy and render and animation oriented. No one should
    have to worry about technical details about how to interact with the OS, filesystem, etc.
    - The `app` module contains pretty much everything to bootstrap a demo quickly.
    - The `app::demo` module contains a trait to implement on a type which objects are used as
      demos. Such an object will react to the window getting resized (for debug purposes) and will
      draw something at the screen at a given time. Also, such a type will be created by passing it
      a `warmy` store, so that it’s easy to get scarce resources.
    - The `app::runner` module contains several demo runners that can deal with `T: Demo` and place
      a context of execution. For instance, with a debug runner, a freefly camera is available by
      hitting a given key, some information are written in the default framebuffer, etc.
    - In order to implement `Demo::render`, people will have to use various types and combine them
      to produce a result. Types like `ViewportRenderer` will need to be sketched up.
    - Especially, most of the types will need to reference `render::Block`. A render block is a
      logical computation (it has inputs and outputs) that abstracts the concept of a shader into
      smaller, more grained and general parts. The idea is that combining render blocks will yield
      something similar to a shader. A render block has information about its inputs, outputs and
      contains GLSL code that links them altogether.
