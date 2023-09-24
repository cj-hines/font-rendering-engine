# Checkpoint draft

## Summary

As of the time of this milestone, we have completed much of the groundwork needed for the final rendering system. This includes reformatting the data from our TTF parsing libary into something more usable for our project, setting up an SDL2 renderer, and establishing the framework for how we will evaluate whether a point is within the drawn portion of a character. Under the amended schedule provided below, we still need to complete:

- The ray-curve intersection formula
- The ray-line intersection formula
- Supersampling
- Performance metrics

## Preliminary results

TODO put some renders here

## Reflection on progress

We have run into some obstacles with maintaining the schedule laid out in our proposal, with single-character rendering still in-progress. Much of this delay came from the difficulties of adapting to the quirks of Rust (e.g. ownership) and the Rust community (e.g. having awful documentation). Eventually we were able to find barebones examples of ttf_parser and rust-sdl2 in use that we could heavily adapt to the purposes of our final project.

The second major obstacle to the single-character rendering implementation is the vagueness of existing documentation. Apple's documentation, the authoritative source for the TTF format, is full of insufficient explanations in essential equations and critical implementation details. The formula to convert from abstract font units to pixels on screen, which is necessary for creating clear renders, has some details which we have had to approximate via trial-and-error. More importantly, we have struggled to find a functional algorithm for Bézier curve-ray intersection that provides (1) whether the ray intersects the curve between the outermost control points and (2) whether the origin of that ray is to the left or to the right of the curve. Once we find definite answers to these, single-character rendering will take less than a day of work.

## Amended work plan

Next four days: Single-character rendering
Remaining time: Supersampling & metrics

This amended schedule reflects the time we have remaining to work on this project and our estimates of the difficulty of the remaining components.
