# UltraFXR Sound Effect Generator (C#)

UltraFXR generates sound effects for your games.  It's inspired by Dr. Petter's SFXR and Stephen Lavell's BFXR.

As of February 2016, this project is not usable yet.  Currently, all it does is replicate BFXR's output, given parameter files created by BFXR.  You can use the command-line tool as follows:

    UltraFXR.exe input.bfxrsound output.wav

This should produce output roughly identical to BFXR's output.  If it doesn't, file a bug.

## Future plans

* Additional sounds.  At least FM synthesis, since it's good for generating percussive, metallic, and wooden sounds.

* Revamped presets.  Sounds from 16-bit era games are a good target.

* Simple modular architecture.  Not something sophisticated like SuperCollider, but just the ability to connect an acyclic graph of modules.

* Ports / bindings to other languages.  This is being written in C# first because C# is nice for development and testing.  JavaScript and C are the top priority ports, but it may be a while.
