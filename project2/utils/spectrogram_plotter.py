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

# 2. Compute the Short-Time Fourier Transform (STFT)
stft_matrix = librosa.stft(y, n_fft=16384, hop_length=256)

# 3. Convert magnitude spectrum to Decibels (dB)
stft_db = librosa.amplitude_to_db(np.abs(stft_matrix), ref=np.max)

# 4. Plot the spectrogram
plt.figure(figsize=(10, 4))
librosa.display.specshow(stft_db, sr=sr, x_axis='time', y_axis='log', cmap='magma')

# 5. Add visual anchors
plt.colorbar(format='%+2.0f dB')
plt.title(f"{audio_path}")
plt.xlabel('Time (s)')
plt.ylabel('Frequency (Hz)')
plt.tight_layout()

# 6. Display the plot
plt.show()
