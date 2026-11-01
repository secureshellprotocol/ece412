import numpy as np
import matplotlib.pyplot as plt
from scipy.io import wavfile
from scipy.linalg import solve_toeplitz
from scipy.signal import lfilter
import librosa
import librosa.display
import IPython.display as ipd

def main():
    filename = 'codingexample.wav'
    fs, data = wavfile.read(filename)
    data = data.astype(np.float64)
    data = data / np.max(np.abs(data)) * 0.95  # normalize, leave a little headroom

    ipd.display(ipd.Audio(data, rate=fs))

    # Frame/subframe structure, matching the 20ms frames / 5ms subframes described above
    frame_ms, subframe_ms = 20, 5
    frame_len = int(frame_ms*fs/1000)
    subframe_len = int(subframe_ms*fs/1000)
    subframes_per_frame = frame_len // subframe_len
    lpc_order = int(1.25*fs/1000)  # same rule of thumb as in the Linear prediction chapter

    # Pitch (adaptive-codebook) lag search range. Lags shorter than one subframe
    # would need the candidate excitation to be generated recursively, since it
    # would overlap the subframe currently being predicted; we avoid that extra
    # bookkeeping by requiring lag >= subframe_len.
    lag_min = subframe_len            # 5ms, i.e. F0 <= 200Hz
    lag_max = int(fs/80)              # F0 >= 80Hz, matching the range used throughout this book

    # Small fixed grids for quantizing the two gains (5 bits each)
    pitch_gain_levels = np.linspace(0.0, 1.2, 32)
    noise_gain_levels = np.geomspace(1e-3, 2.0, 32)

    n_frames = len(data)//frame_len
    data = data[:n_frames*frame_len]

    excitation_history = np.zeros(lag_max + frame_len)  # the adaptive codebook
    synth_state = np.zeros(lpc_order)                    # LPC synthesis filter memory

    decoded = np.zeros(len(data))
    bits_used = 0
    bits_per_subframe = 8 + 5 + 7 + 1 + 5  # lag, pitch gain, pulse position, pulse sign, noise gain

    for f in range(n_frames):
        frame = data[f*frame_len:(f+1)*frame_len]

        # -- LPC analysis (same convention as in the Linear prediction chapter) ---
        windowed = frame * np.hanning(frame_len)
        autocorr = np.correlate(windowed, windowed, mode='full')[frame_len-1:frame_len+lpc_order]
        autocorr[0] *= 1.0001  # tiny regularisation to keep the Toeplitz system well-conditioned
        b = np.zeros(lpc_order+1); b[0] = 1.0
        a = solve_toeplitz(autocorr, b)
        a = a / a[0]  # A(z) = 1 + a_1 z^-1 + ... + a_p z^-p

        for s in range(subframes_per_frame):
            start = f*frame_len + s*subframe_len
            target = data[start:start+subframe_len]

            # LPC residual of this subframe, using the true past samples as filter memory
            ctx = data[max(0, start-lpc_order):start]
            ctx = np.concatenate((np.zeros(lpc_order-len(ctx)), ctx))
            x = lfilter(a, [1.0], np.concatenate((ctx, target)))[lpc_order:]

            # --- long-term (pitch) prediction: x_F0^*, argmax over candidate lags T ---
            scores = []
            for T in range(lag_min, lag_max+1):
                cand = excitation_history[-T:len(excitation_history)-T+subframe_len]
                scores.append(np.dot(x, cand)**2 / (np.dot(cand, cand) + 1e-9))
            best_T = lag_min + int(np.argmax(scores))
            x_f0 = excitation_history[-best_T:len(excitation_history)-best_T+subframe_len]

            gain_f0 = np.dot(x, x_f0) / (np.dot(x_f0, x_f0) + 1e-9)               # gamma_F0^*
            gain_f0 = pitch_gain_levels[np.argmin(np.abs(pitch_gain_levels - gain_f0))]
            x_prime = x - gain_f0 * x_f0                                          # F0-residual

            # --- noise codebook: simplest case, one signed pulse per subframe ---
            pulse_pos = int(np.argmax(x_prime**2))
            pulse_sign = np.sign(x_prime[pulse_pos]) or 1.0
            x_noise = np.zeros(subframe_len)
            x_noise[pulse_pos] = pulse_sign

            gain_noise = x_prime[pulse_pos] * pulse_sign                          # gamma_noise^*
            gain_noise = noise_gain_levels[np.argmin(np.abs(noise_gain_levels - gain_noise))]

            # --- quantized excitation and reconstruction, x_hat^* ---
            excitation = gain_f0*x_f0 + gain_noise*x_noise
            excitation_history = np.concatenate((excitation_history[subframe_len:], excitation))

            synth_out, synth_state = lfilter([1.0], a, excitation, zi=synth_state)
            decoded[start:start+subframe_len] = synth_out

            bits_used += bits_per_subframe

    decoded = np.clip(decoded, -1.0, 1.0)

    duration_s = len(data)/fs
    bitrate_kbps = bits_used/duration_s/1000
    print(f"Encoded {duration_s:.2f}s of speech at {bitrate_kbps:.1f} kbit/s "
          f"(excluding the LPC coefficients themselves, which we did not quantize)")

    data_f = data.astype(np.float32)
    decoded_f = decoded.astype(np.float32)

    plt.figure(figsize=(8,2))
    librosa.display.waveshow(data_f, sr=fs)
    plt.title("Original speech sample")
    plt.xlabel("Time (s)")
    plt.ylabel("Amplitude")
    plt.show()
    ipd.display(ipd.Audio(data_f, rate=fs))

    plt.figure(figsize=(8,2))
    librosa.display.waveshow(decoded_f, sr=fs)
    plt.title(f"CELP-decoded speech sample ({bitrate_kbps:.1f} kbit/s)")
    plt.xlabel("Time (s)")
    plt.ylabel("Amplitude")
    plt.show()
    ipd.display(ipd.Audio(decoded_f, rate=fs))

    wavfile.write("codedsounds.wav", fs, decoded)



if __name__ == "__main__":
    main()
