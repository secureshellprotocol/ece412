# CELP codec

base case by Tom Bäckström, Aalto University. Code in this directory is
CC-BY-4.0. see credits

## usage 

make sure you have your uv environment set up

```bash
uv sync
```

To re-encode a wav file:

```bash
uv run main.py <audio_in.wav> [--lpc_quantization] [--colored] [--no-graphs]
```

It will write a file to `output.wav`

# credits

Tom Bäckström, Okko Räsänen, Abraham Zewoudie, Pablo Pérez Zarazaga, Liisa Koivusalo, Sneha Das, Esteban Gómez Mellado, Mariem Bouafif Mansali, Daniel Ramos, Sudarsana Kadiri, Paavo Alku, and Mohammad Hassan Vali “Introduction to Speech Processing”, 2nd Edition, 2022. URL: https://speechprocessingbook.aalto.fi, DOI: 10.5281/zenodo.6821775

# ai disclosure

chatterbox.ee ~~acted as an autocomplete~~ generated code in this repo to modify
the base case
