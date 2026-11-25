---
title: "Fluid Simulation in Rust"
date: 2026-11-26T00:00:00+09:00
draft: false
tags: ["Rust", "macroquad", "WebAssembly", "Fluid Simulation"]
categories: ["Notes"]
description: "Interactive 2D and 3D fluid simulations made with Rust and macroquad, compiled to WebAssembly for the browser."
---

{{< katex >}}

I made two fluid simulations in Rust with macroquad: a canvas of swirling dye and a box filled with colorful particle trails.
Both are compiled to WebAssembly, so you can try them in your browser.
I'm happy with how smoothly the simulations run in the browser. Rust and WebAssembly are fast enough to calculate and draw the flow interactively, which makes it fun to experiment with.

## Try it in 2D

Click **Start simulation**, then left-drag to add dye and stir the flow.

{{< fluid mode="2d" >}}

## Try it in 3D

Left-drag to stir the flow and release particles. Hold **Shift** while dragging to rotate the camera, and use the **scroll wheel** to zoom.

{{< fluid mode="3d" >}}

A mouse and keyboard are recommended. Running both simulations at the same time may slow things down on some devices.

## How I built it

The simulations are based on the incompressible Navier–Stokes equations. The velocity field \(\mathbf{u}(\mathbf{x},t)\) describes the direction and speed of the flow at position \(\mathbf{x}\) and time \(t\):

$$
\begin{aligned}
\frac{\partial\mathbf{u}}{\partial t} + (\mathbf{u}\cdot\nabla)\mathbf{u}
&= -\frac{1}{\rho}\nabla p + \nu\nabla^2\mathbf{u} + \mathbf{f}, \\
\nabla\cdot\mathbf{u} &= 0.
\end{aligned}
$$

Here, \(p\) is pressure, \(\rho\) is constant density, \(\nu\) is kinematic viscosity, and \(\mathbf{f}\) is external force per unit mass. The simulations use simplified numerical approximations of these equations.

I represent the fluid on a regular grid, with an extra layer of cells around the edge for the boundary conditions. Each cell stores a velocity, and dragging the mouse adds motion near the cursor.

Two steps shape the flow: advection and pressure projection.

**Advection** carries quantities such as dye and velocity along with the flow. To find the new value of a quantity \(a\) in each cell, I trace backward through the flow and interpolate the previous values at that position:

$$
a(\mathbf{x},t+\Delta t)
\approx a\!\left(\mathbf{x}-\Delta t\,\mathbf{u}(\mathbf{x},t),t\right).
$$

**Pressure projection** keeps the flow approximately incompressible. The condition \(\nabla\cdot\mathbf{u}=0\) means that, locally, as much fluid flows in as flows out. Starting with a tentative velocity \(\mathbf{u}^{*}\), I solve for a scaled pressure correction \(\phi\) and subtract its gradient:

$$
\nabla^2\phi=\nabla\cdot\mathbf{u}^{*},
\qquad
\mathbf{u}=\mathbf{u}^{*}-\nabla\phi.
$$

Taking the divergence of the second equation shows how the correction cancels the divergence. The solver only runs for a limited number of iterations, so the result is approximate.

