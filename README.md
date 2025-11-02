# Musiq - The Automated Music Playing Software

## Info
This program was developed (and is in development) for the audio technics
of my high school; the leadership of which, I have realized in the summer.

There, an Ubuntu server runs this program, which is connected to the amplifier
of the school's speakers. These are powered on every morning and turned off
every afternoon.

The default break starting times are so oddly specific, because we still use
traditional bells, and that system looses a minute every half a year.

The main purpose of this software is to ease the burden of selecting and
playing a song every break. This is done by collecting the students' and
teachers' favourite songs, and deciding whether to add them to the program's
list of songs. The software from this point on, plays a random song after
every period _(when that break is enabled)_, keeping track of previously
played music and only repeating, when all of them was played.

## Installing
The recommended way of installing is to build from source on the target machine.
This can be done with 'Cargo', Rust's package manager.

To start, one must have Rust [installed](https://rust-lang.org/tools/install/).
After downloading the source code, navigate to its folder _(the one in which `Cargo.toml`
lies)_ and run `cargo build --release` for building (the binary will be located in
`/target/release/`), or `cargo run --release` for running the program.

### Note
_Installing from source is advised, because of the sound handling library used, which
does not like to be ported_

### Tested environments
The software is developed on Windows 11 (x86_64) and used on Ubuntu 24.04 LTS (x86_64), but
is tested to work on Ubuntu 24.04 LTS (aarch64).\
Before using on Linux environments, other components may be required to be installed (usually
indicated by the compiler)

## Use
The program can be started with `./musiq [address:port]`, where the address and
the port for hosting the web UI _may_ be specified. The default is `0.0.0.0:80`.

For mainly debugging reasons, there are some command line switches:
- `-E` enables all songs
- `-D` disables all songs
- `-R` resets the played status of all songs

Configuring the program as well as adding songs and events can be done using the web UI,
which can be found on the previously specified webpage.

See `ENVVARS.md` for accepted compile-time and runtime environment variables

## Note
Currently, the program can only handle `mp3` audio files and supports a handful of audio
devices. Usually the default system device works.

When the program starts on Linux environments, a loud pop could be heard. This seems to be
the OS's fault.

This program is still in active development, changes could occur anytime _(see `TODO.md`
for potential improvements)_.