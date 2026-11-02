#!/bin/bash -x
ffmpeg -i $1 -c:a pcm_s16le output_file.wav
