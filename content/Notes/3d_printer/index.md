---
title: "3D Printing Isty"
date: 2026-08-06T03:00:00+09:00
draft: false
tags: ["Blender", "3D Printing", "Isty"]
categories: ["Notes"]
description: "Modeling the mascot character of my graduate school in Blender and printing it on a Bambu Lab X2D."
---

My graduate school has a very cute mascot character named [Isty-kun](https://www.i.u-tokyo.ac.jp/isty_e.shtml).\
His name comes from **I**nformation **S**cience and **T**echnolog**y**, and he is a small robot who protects safety and security in the information society.

![The Isty mascot character: a yellow egg-shaped robot with an antenna](Isty.png "Isty, the mascot of the Graduate School of Information Science and Technology.")
{style="max-width: 300px; margin-inline: auto;"}

I liked him so much that I decided to model him in Blender and 3D print him, just for fun.

## Modeling in Blender

The whole character is built from two kinds of objects: one solid body, and a set of curves that provide every line.
His structure is simple enough that putting him together in Blender took almost no time.

![Blender viewport showing the Isty model with its outliner and material settings](blender.png "The model in Blender. The body is a single mesh; the antenna, arms, eyes, and mouth are curve objects.")

Here is the exported model, embedded directly into this page. \
Drag to rotate him, and scroll to zoom. 
**See how cute he is!**

{{< model src="isty.glb" alt="Interactive 3D model of the Isty character" >}}

## Slicing in Bambu Studio

Once the shapes were finished, I exported the whole scene as a single OBJ file and opened it in [Bambu Studio](https://bambulab.com/en/download/studio), the slicer for my [Bambu Lab X2D](https://store.bambulab.com/products/x2d).\
Bambu Studio was easy to install, and it sliced him with almost no configuration from me.

![Bambu Studio showing the Isty model on the build plate next to a prime tower](bamboo_studio.png "Isty on the build plate.")

The part I was worried about was his floating antenna, which has nothing underneath it to rest on.
In the end I did not have to do anything: the slicer found the overhangs by itself and generated supports under the antenna and under both arms.

![Sliced preview in Bambu Studio with generated support structures under the antenna and arms](bamboo_studio_support.png "The sliced result. The cone-shaped structures under the antenna and the arms are the supports added automatically.")

## Printing

Sadly, the yellow filament was out of stock, so we had to print him in black and white instead of his own colour. \
The X2D has a built-in camera, so the print recorded itself.

{{< video src="isty.mp4" ratio="16/9" controls="true" muted="true" loop="true" caption="Timelapse from the printer's built-in camera. The prime tower grows alongside the model." >}}

I am really happy to have a printed Isty. I expected more trouble with the supports, but Bambu Studio handled them automatically.\
I might get a 3D printer of my own someday.
