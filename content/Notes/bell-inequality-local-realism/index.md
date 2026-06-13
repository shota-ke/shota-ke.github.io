---
title: "Bell Inequality, Local Realism, and What We Can Build"
date: 2026-06-13
draft: false
tags: ["Quantum Information", "Bell Inequality", "Local Realism", "Entanglement"]
categories: ["Notes"]
description: "A short note on Bell's inequality, local realism, interpretation, and why I see it as a starting point for quantum information engineering."
---

{{< katex >}}

Bell's inequality is one of those topics that appears very naturally in courses on quantum mechanics or quantum information.
The calculation is standard.
The conclusion is much less standard.

It is tempting to summarize Bell experiments by saying that "local realism is violated."
That sentence is not wrong, but it hides the part that I find interesting.
What exactly is being violated?
Does the experiment prove that locality is false?
Does it prove that realism is false?
Or is the right conclusion something more careful?

This note is my attempt to organize that question.
I am not trying to settle the interpretation of quantum mechanics.
Rather, I want to separate three layers that are often mixed together:

1. the mathematical statement of Bell's inequality,
2. the interpretational choices one can make after its violation,
3. the engineering attitude of using nonclassical correlations as a resource.

My conclusion is simple.
Bell experiments rule out a certain classical picture of correlations, but they do not by themselves choose a unique interpretation of quantum mechanics.
For me, the most productive next step is not to decide the final metaphysical meaning of the result.
It is to ask what can be built from the kind of correlation that quantum mechanics makes possible.

## Local Realism

The classical picture usually discussed in this context is called **local realism**.
Roughly speaking, it combines two ideas.

**Realism** says that measurement outcomes reveal values that were already fixed, perhaps by some hidden variable we do not know.
If Alice can choose between two measurements, then the system already carries the answers she would obtain for those possible choices.

**Locality** says that the outcome at one location is not instantly changed by the choice of measurement performed at a distant location.
If Alice and Bob are far apart, Bob's result should not depend on which measurement Alice decides to perform at the last moment.

Both ideas sound reasonable.
It feels natural to think that measurement reveals a property the system already had.
It also feels natural to think that distant choices do not immediately affect local outcomes.

Bell's theorem shows that when these intuitions are made precise, they imply a bound on possible correlations.
In a compact form,

$$
\text{locality} + \text{realism} \Longrightarrow \text{Bell inequality}.
$$

But experiments violate Bell inequalities.
Therefore the combined package cannot be kept as it is.
At least one part must be abandoned or modified.

The important point is that a Bell experiment alone does not tell us which word, "locality" or "realism," is the one to discard.
That is already an interpretational choice.

## The CHSH Game

The cleanest example is the CHSH inequality.
Imagine that Alice and Bob each receive one part of a shared physical system.
Alice can choose one of two measurements, `A0` or `A1`.
Bob can choose one of two measurements, `B0` or `B1`.
Each measurement returns either `+1` or `-1`.

The CHSH expression is

$$
S =
E(A_0B_0) +
E(A_0B_1) +
E(A_1B_0) -
E(A_1B_1),
$$

where `E(AiBj)` is the average product of Alice's and Bob's outcomes when those measurement settings are chosen.

If the outcomes are explained by a local hidden-variable model, then

$$
|S| \leq 2.
$$

This is not a quantum-mechanical statement.
It is a constraint on a classical kind of explanation.
The hidden variable may be random and unknown to us, but in each run it is supposed to determine what Alice and Bob would answer for each possible measurement setting.
Also, Alice's answer is allowed to depend on Alice's local setting, but not on Bob's distant choice, and vice versa.

Quantum mechanics allows

$$
|S| \leq 2\sqrt{2},
$$

and suitable entangled states and measurement bases can reach this larger value.

The point is not just that `2 sqrt(2)` is larger than `2`.
The point is that quantum mechanics predicts correlations that cannot be reproduced by pre-existing local instructions.

## What Is Actually Violated?

When people say that "local realism is violated," it can sound as if quantum mechanics has simply proved that nature is nonlocal, or that particles communicate faster than light.
That is too quick.

In the CHSH setting, what is ruled out is a picture where the pair already carries a complete local instruction sheet before the measurements happen.
That instruction sheet would have to determine what Alice would answer for `A0` and `A1`, and what Bob would answer for `B0` and `B1`.
Alice's answers would not depend on Bob's later choice, and Bob's answers would not depend on Alice's later choice.

Quantum correlations do not fit into that picture.

So Bell violation does not directly say:

- locality alone is false,
- realism alone is false,
- or faster-than-light signaling is possible.

It says that the specific locally realistic model used in the derivation cannot describe nature.
If one wants to keep realism, one must give up or modify locality.
If one wants to keep locality, one must give up or modify the idea of pre-existing values.
If one wants to avoid both words, one can take a more operational attitude and treat quantum mechanics as a rule for predicting observed correlations.

This does not mean that Bell violation lets us send a message faster than light.
Alice cannot choose her outcome.
Bob's local statistics do not reveal Alice's measurement choice by themselves.
The violation appears only when Alice and Bob later compare their data and estimate the correlations.

For me, the most useful statement is that entanglement can produce correlations stronger than anything obtainable from shared classical randomness and local response functions.
That is less dramatic than "reality is broken," but it is much clearer.

## Ways to Read the Result

There are several ways to read this result.
The following are not exhaustive, but they show why I do not think Bell experiments alone force one unique philosophical conclusion.

### Give Up Realism About Pre-Existing Values

One response is to reject the idea that all possible measurement outcomes were already fixed before measurement.
In this view, it is not meaningful to say that Alice's particle had definite answers for both `A0` and `A1` before Alice chose which measurement to perform.

This is close in spirit to many textbook or Copenhagen-like presentations.
The quantum state is not treated as a list of hidden pre-existing values.
Instead, it gives probabilities for outcomes of actually performed measurements.

From this viewpoint, Bell violation is not evidence that a signal traveled between Alice and Bob.
It is evidence that the classical habit of assigning simultaneous definite values to unmeasured alternatives does not work.

### Keep Realism, Give Up Locality

Another response is to keep a realist picture but allow nonlocal structure.
Bohmian mechanics is the standard example.
Particles have definite positions, but their dynamics depends on the wavefunction in a way that is explicitly nonlocal.

This kind of theory can reproduce quantum predictions, including Bell inequality violations.
It says, in effect, that the world can be realist, but not locally realist.
This does not automatically allow faster-than-light communication, but the underlying description is not local in the classical sense.

### Keep the Formalism, Avoid a Single Outcome Story

Many-worlds-like interpretations take a different route.
They keep the unitary evolution of the wavefunction as the fundamental process and do not add hidden variables or a physical collapse.

In that picture, measurement does not select one outcome from a list of pre-existing values.
Instead, the observer becomes entangled with the measured system, and different branches contain different outcomes.

Whether this should be called "realist" depends on what one means by realism.
The wavefunction is often treated as real, but individual measurement outcomes are not pre-assigned in the classical hidden-variable sense.
So Bell's theorem is avoided not by adding superluminal signals, but by rejecting the classical single-outcome hidden-variable structure assumed in the inequality.

## The Mathematics Does Not Need One Interpretation

There is another point that matters to me.
The mathematical formalism of quantum mechanics works independently of which interpretation one prefers.

For example, when we calculate a Bell inequality violation, we choose a state, choose measurement operators, compute expectation values, and compare the resulting correlations with the classical bound.
The calculation does not require us to first decide whether the wavefunction is a physical object, a state of knowledge, a branch structure, or something else.

The same attitude appears in more modern diagrammatic methods.
The [PennyLane introduction to the ZX-calculus](https://pennylane.ai/demos/tutorial_zx_calculus/) describes ZX-calculus as a graphical language for reasoning about quantum computations and circuits.
In that framework, quantum processes are represented as diagrams, and valid transformations are given by rewriting rules that preserve the underlying linear map.

That is a useful contrast with the interpretation problem.
At the interpretational level, one may argue about what the quantum state means.
At the mathematical and engineering level, one can manipulate circuits, simplify diagrams, optimize gates, verify equivalences, and compile computations.

In other words, the formalism gives us a stable operational playground even when the metaphysical story is not unique.
This is one reason I am comfortable saying both things at once:
Bell experiments do not by themselves choose a single interpretation, but Bell violation is still a concrete and useful fact.

## From Understanding Nature to Engineering It

There is a common way to end a discussion of Bell's inequality:

> The universe is not locally realistic.

I understand the appeal of that sentence.
Bell's theorem really does force us to give up a simple classical picture.

In that sense, Bell's inequality is certainly a philosophical landmark.
But for me, it is more than that.
It is also one of the starting points of quantum information engineering.
Instead of only asking what these correlations say about the nature of reality, quantum information asks how they can be used.

The world may not fit into the local-realist model.
For me, the next question is not only what that says about the deep structure of nature.
It is what we can build because of it.
