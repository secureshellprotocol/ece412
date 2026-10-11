# Voice modulator toolkit

James Ryan, Prof. Colonel, Speech and Audio Processing. Fall 2026. 

Tested on Fedora Linux

## Getting started

### Prereqs

Need to have [Rust](https://rust-lang.org/install.html) installed, and need to
have a version of python available.

I recommend using [uv](https://docs.astral.sh/uv/getting-started/installation/)
installed and managing your Python 3 environment.

You need a local copy of [Praat](https://www.fon.hum.uva.nl/praat/) to run the
PSOLA algorithms. This needs to be in your PATH, so Python / parselmouth can
utilize it

## Rust stuff

Implements the Graphic EQ & Sampling based stretcher / pitch shifter

```
cargo run --release -- [path_to_wav_file]
```
### Block configurations

As of Sunday, we have:
* 'equalizer' - Graphic EQ that has three filters (low shelf, bell, high shelf)
* 'reverb' - Time-delay reverb. Currently set to be a constant timeout, so may
    fail on very short signals
* 'pitch' - Upsample/downsample based pitch shifting, with optional
    alias/imaging filters
* 'compressor' - Dyanmic range compressor for limiting max/min amplitudes
* 'amplifier' - Signal amplifier

Most of these can be adjusted, see the `config/` directory.

### Running blocks

Jobs are defined in `config/ordering.yml`. You specify the names of the blocks
to run in order. If there is an invalid block name, the job will attempt to
continue anyway.s 

### Regenerating configs

Mess them up real bad? or want some "sane" defaults? Run:

```
cargo test
```

and it should deposit new generated configs in `./config`

## Python stuff

Implements the PSOLA demos (wraps Praat), and some helper scripts.

To pull dependencies:

```
uv sync
```

Then specify the files under `src/` to run. Eg:

```
uv run src/spectrogram_plotter.py awesome.wav &
```


# gen ai disclosure

written using ai assistance (nvidia/Gemma-4-26B-A4B-NVFP4, running on cooper
union's gpu cluster). it didnt really do so well though >.>
