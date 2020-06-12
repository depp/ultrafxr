# UltraFXR

**Status:** Not ready for use. In active development as of March 2020.

UltraFXR - Ultra Fun Expression! Generate sound effects and music for your demo or game.

MIT licensed, see [LICENSE.txt](LICENSE.txt) for details.

## Current Status

Not yet ready for use. In active development as of March 2020. Currently, this system has near feature parity with a system made for JS13K 2019, except for the ability to render an entire musical score. The documentation is nonexistent, but the error messages are fairly good.

You can see sample audio programs in the examples directory. The compiler in the `rust` directory can render these as audio files.

## Goals

UltraFXR has two main goals: replace BFXR and generate audio for demos.

### Replace BFXR

BFXR is old and written in Flash, and yet it’s a critical tool for game jams. Flash’s end of life, December 2020, is coming soon! At that point, the web version of BFXR will be cumbersome to use. We can provide a drop-in replacement for BFXR. BFXR is open source, and we can replicate the sounds it makes in our own engine.

We don’t just want to replace BFXR, however. UltraFXR should provide new templates and macros to generate new types of sounds.

### Audio for Demos

UltraFXR is based on tools that were originally written for JS13K. A lot of the design is centered around the ability to synthesize audio with a small amount of code. This includes both sound effects and musical instruments, with the abilty to play back a score.

## Road to 1.0

- The compiler should run on Linux, macOS, Windows, and it should run in the browser with WebAssembly.

- Right now, functions and other operators in the Lisp code are implemented in Rust. They should be rewritten mostly in Lisp, with only the lower-level base operators written in Rust.

- Provide a version of the synthesis engine in WebAssembly, optimized for speed.

- Write documentation.

- Create a web UI for creating sounds with knobs and sliders, like BFXR.

- Clean up and expand selection of operators.

- Port BFXR’s sounds.

## Future Thoughts

- Add support for music.

- Demoscene packer, which emits a small JavaScript or WebAssembly bundle from a selection of sounds.

- Better sounding filters.

- Reverb.

- Physical modeling synthesis.

- Investigate making a “high-quality” version of the engine with techniques like supersampling.

- Live playback via MIDI.

- Create Audio Units or VSTs from programs.

- More sophisticated in-browser editor features.

## Building

For development, it is recommended to use `-Werror` for C code. This can be configured with a `.bazelrc.user` file:

    build --copt -Werror
