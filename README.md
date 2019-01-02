# AzureOST
### An OST ripper for FFXIV written in Rust
#### Why should I use AzureOST?

- **It's *heckin* fast** - Written in Rust and designed around speed, AzureOST
completes its job in a fraction of the time as other exporters like SaintCoinach.
- **It does the loop-the-loop** - AzureOST parses Square Enix's embedded tags
for seamless looping and loops the output appropriately. It also applies a
fade-out effect, because no one likes a hard cut-off to their music.
- **It splits hairs** - Some songs in FFXIV's data files are encoded as four or
six channels for 2 or 3 separate songs. AzureOST is the only exporter that
splits these files so you can listen to your techno Binding Coil battle theme
without having to listen to the resting theme at the same time.
- **It takes it easy** - AzureOST doesn't do what it doesn't have to. It can
save a manifest of all the tracks it has seen and only exports new/changed ones.
- **It threads the needle** - AzureOST doesn't just ignore the power of
modern hardware. It has the capability to use as many threads as you want it to.
(By default it'll use the number of logical cores on your system).
- **It does what you want** - AzureOST can export to OGG/Vorbis or MP3. (To
export to MP3 you'll have to compile with the `lamemp3` feature enabled, and
have access to libmp3lame on your system.)
- **It doesn't discriminate** - AzureOST is designed to be cross-platform.
Simply compile it using Cargo, and you're off to the races!
---
#### How to use AzureOST
##### Front-end
AzureOST is a back-end library that handles the possible exporting /
manifest options, that needs a front-end executable to function.
This can come in a variety of forms. One such front-end I wrote to
demonstrate this is 
[`azureost-cli`](https://github.com/CerulanLumina/azure-ost-cli),
which serves as a basic command-line interface for AzureOST.

---
#### What's not ready yet?

- The codebase has been cleaned some with the refactor into a module system,
but there's always more room for documentation.
- Probably more things that I can't remember right now

---
#### Speed

**Note**: The test below is in reference to the original version of AzureOST.
The refactored system will be tested when I am able to test on the same hardware.

As stated above, `azure-ost` aims to be *fast*. I think I have succeeded in this
endeavor. On a Ryzen 5 1600X (12 threads @ 3.6 GHz), the entire BGM list is
saved into a manifest, decoded once from SCD, and again from OGG/Vorbis, looped,
encoded into MP3 and exported in under 8 minutes (`time`: `real 7m48.816s`).
I also think there's more optimization possible, once I clean and refactor the
code. 

---
#### Modules


- azureost-core-rs - Core functionality for azureost-rs
  - azureost-cli - Command-line interface for the application
  - azureost-jni - (*Planned*) Java Native Interface library for the application
    - azureost-jgui - (*Planned*) GUI written in Java (possibly) that executes calls to the JNI (then to the core)


---
### Background

This project aims to create a command-line utility, written in Rust, that is
able to export the background musicfrom FFXIV. The goal in undertaking this
process is to create a faster, cross-platform version, with fewer dependencies
than an earlier, unreleased version written in Java. (This Java version
internally used the SaintCoinach CMD utility, available here:
https://github.com/ufx/SaintCoinach, VGMStream to loop the songs, as well as the
FFmpeg installed in the user's PATH. Because SaintCoinach is written in C#/.NET
it was only possible to use this application on Windows environments.

With this program, however, I aim to handle all the exporting, decoding,
converting, and looping myself. (Though of course I'll use standard library for
basic operation and crates for things like converting from OGG to MP3. As long
as they're cross platform.) 

This project contains within it a references a library crate called
[sqpack_blue](https://github.com/CerulanLumina/sqpack-blue), which is the basic
interface for reading FFXIV's data files. On release, it should have the
ability to export raw files (without decoding from Square Enix's format),
export data sheets (which *are* decoded from Square's EXHF/EXDF files), and
export SCD files, which can be decoded into WAV/MSADPCM or OGG/Vorbis formats
(the specific SCD file selected determines which can be decoded).
