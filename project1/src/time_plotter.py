import librosa
import librosa.display
import matplotlib.pyplot as plt
import numpy as np
import sys

if len(sys.argv) == 1:
    print("Usage: {} [wav to plot]", sys.argv[0])

# 1. Load the audio file (automatically handles mono conversion and resampling)
# Use sr=None to keep the original sample rate of your file
audio_path = sys.argv[1]
y, sr = librosa.load(audio_path, sr=None)

# 4. Plot the spectrogram
plt.figure(figsize=(10, 4))
librosa.display.waveshow(y, sr=sr)

# 5. Add visual anchors
plt.title(f"{audio_path}")
plt.xlabel('Time (s)')
plt.ylabel('Amplitude')
plt.xlim([.08, .38])
plt.tight_layout()

# 6. Display the plot
plt.show()
