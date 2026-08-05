---
title: "3D Printing Isty"
date: 2026-08-06T03:00:00+09:00
draft: false
tags: ["Blender", "3D Printing", "Isty"]
categories: ["Notes"]
description: "Modeling the mascot character of my graduate school in Blender and printing it on a Bambu Lab X2D."
---

My graduate school has a very cute mascot character named [Isty-kun](https://www.i.u-tokyo.ac.jp/isty_e.shtml).
His name comes from **I**nformation **S**cience and **T**echnolog**y**, and he is a small robot who protects safety and security in the information society.

![The Isty mascot character: a yellow egg-shaped robot with an antenna](Isty.png "Isty, the mascot of the Graduate School of Information Science and Technology.")
{style="max-width: 300px; margin-inline: auto;"}

I liked him so much that I decided to recreate him in Blender and bring him into the real world with a 3D printer.
This was a purely personal project, made just for fun.

## Modeling in Blender

The whole character is built from two kinds of objects: one solid body, and a set of curves that provide every line.
His structure is simple enough that putting him together in Blender took almost no time.

![Blender viewport showing the Isty model with its outliner and material settings](blender.png "The model in Blender. The body is a single mesh; the antenna, arms, eyes, and mouth are curve objects.")

Here is the exported model, embedded directly into this page.
Drag to rotate him, and scroll to zoom. \
**See how cute he is!**

{{< model src="isty.glb" alt="Interactive 3D model of the Isty character" >}}

## Slicing in Bambu Studio

Once the shapes were finished, I exported the whole scene as a single OBJ file and opened it in [Bambu Studio](https://bambulab.com/en/download/studio), the slicer for my [Bambu Lab X2D](https://store.bambulab.com/products/x2d).
Bambu Studio was easy to install, and it sliced him with almost no configuration from me.

![Bambu Studio showing the Isty model on the build plate next to a prime tower](bamboo_studio.png "Isty on the build plate.")

The part I was worried about was his floating antenna, which has nothing underneath it to rest on.
In the end I did not have to do anything: the slicer found the overhangs by itself and generated supports under the antenna and under both arms.

![Sliced preview in Bambu Studio with generated support structures under the antenna and arms](bamboo_studio_support.png "The sliced result. The cone-shaped structures under the antenna and the arms are the supports added automatically.")

## Printing

The X2D has a built-in camera, so the print recorded itself.

{{< video src="isty.mp4" ratio="16/9" controls="true" muted="true" loop="true" caption="Timelapse from the printer's built-in camera. The prime tower grows alongside the model." >}}

## Closing thoughts

I am really happy that he now exists in the physical world.
What surprised me most was how little friction there was along the way: designing a personal object and actually holding it in my hand turned out to be easy from end to end.
3D printers are not that expensive anymore, so I am starting to think that I want one of my own someday.

Photo-based 3D reconstruction and diffusion-based generative models are both moving quickly right now.
Between tools like those and a printer sitting on your desk, it feels like we are getting close to being able to build whatever we can imagine.
