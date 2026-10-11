import numpy as np
import soundfile as sf

# 1. Define audio parameters
sample_rate = 44100  # Standard sample rate (e.g., 44.1 kHz)
duration = 1.0       # Duration in seconds
num_samples = int(sample_rate * duration)

# 2. Generate a flat zero array
impulse_response = np.zeros(num_samples, dtype=np.float32)

# 3. Create the 0 dBFS peak impulse (Kronecker delta)
# A sample value of 1.0 exactly equals 0 dB Full Scale (0 dBFS)
impulse_response[0] = 1.0

# 4. Save to a WAV file using soundfile
output_filename = "zero_db_impulse.wav"
sf.write(output_filename, impulse_response, sample_rate)

print(f"Successfully generated '{output_filename}' at {sample_rate}Hz.")
