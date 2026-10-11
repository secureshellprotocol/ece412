# Voice modulator toolkit

James Ryan, Prof. Colonel, Speech and Audio Processing. Fall 2026. 

Tested on Fedora Linux

## Getting started

### Prereqs

Need to have [Rust](https://rust-lang.org/install.html) installed, and need to
have a version of python available.

I recommend using [uv](https://docs.astral.sh/uv/getting-started/installation/)
installed and managing your environment.

You need a local copy of [Praat](https://www.fon.hum.uva.nl/praat/) to run the
PSOLA algorithms. This needs to be in your PATH, so Python / parselmouth can
utilize it

### Rust

Implements the Graphic EQ & Sampling based stretcher / pitch shifter

```
cargo run --release
```

### Python

Implements the PSOLA demos (wraps Praat)

To pull dependencies:

```
uv sync
```

Then specify the files under `src/` to run. Eg:

```
uv run src/spectrogram_plotter.py awesome.wav &
```

## Regenerating configs

Mess them up real bad? or want some "sane" defaults? Run:

```
cargo test
```

and it should deposit new generated configs in `./config`
