# CC-BY-4.0 Tom Backstrom
# modified by james ryan for ece412 - speech and audio processing, cooper union

import numpy as np
from numpy import transpose
import matplotlib.pyplot as plt
from scipy.io import wavfile
from scipy.linalg import solve, solve_toeplitz, inv, toeplitz
from scipy.signal import lfilter
import sys
import librosa
import librosa.display
import IPython.display as ipd

## all of my base case extenstions will be introduced as external modules
from noise_filter import ColoredNoiseFilter
from utils.autocorr import estimate_f0_robust
from utils.speex import get_speex_weighting_coeffs
from utils.quantize_lpc import quantize_lpc

def main(
    filename,
    apply_color_filter,
    lpc_quantization,
    no_graphs,
):
    fs, data = wavfile.read(filename)
    data = data.astype(np.float64)
    data = data / np.max(np.abs(data)) * 0.95  # normalize, leave a little headroom

    ipd.display(ipd.Audio(data, rate=fs))

    # Frame/subframe structure
    # each frame is used to calculate the autocorr matrix (f0)
    frame_ms, subframe_ms = 20, 5
    frame_len = int(frame_ms*fs/1000)
    subframe_len = int(subframe_ms*fs/1000)
    subframes_per_frame = frame_len // subframe_len
    # assumption is: vocal tract is 17cm, and speed of sound in vocal tract is 350m/s
    # M = (2 * f_s * L) / c which approx (1.25 / 1000) * f_s
    lpc_order = int(1.25*fs/1000)
    print(f"LPC Order {lpc_order}")

    # Pitch (adaptive-codebook) lag search range. Lags shorter than one subframe
    # would need the candidate excitation to be generated recursively, since it
    # would overlap the subframe currently being predicted; we avoid that extra
    # bookkeeping by requiring lag >= subframe_len.
    lag_min = subframe_len            # 5ms, i.e. F0 <= 200Hz
    lag_max = int(fs/80)              # F0 >= 80Hz, matching the range used throughout this book

    # Small fixed grids for quantizing the two gains (5 bits each)
    pitch_gain_levels = np.linspace(0.0, 1.2, 32)
    noise_gain_levels = np.geomspace(1e-3, 2.0, 32)

    # chunk out into frames using frame_ms
    n_frames = len(data)//frame_len
    data = data[:n_frames*frame_len]

    excitation_history = np.zeros(lag_max + frame_len)  # the adaptive codebook
    synth_state = np.zeros(lpc_order)                    # LPC synthesis filter memory

    decoded = np.zeros(len(data))
    bits_used = 0

    # if the lpc bitrate is too low, it can just fail to do anything
    lpc_bits = 16
    if lpc_quantization:
        bits_per_subframe = 8 + 5 + 7 + 1 + 5 + lpc_bits
    else:
        bits_per_subframe = 8 + 5 + 7 + 1 + 5
    # lag, pitch gain, pulse position, pulse sign, noise gain, lpc quanitzation

    for f in range(n_frames):
        frame = data[f*frame_len:(f+1)*frame_len]

        # -- LPC analysis (same convention as in the Linear prediction chapter) ---
        # autocorr describes periodic structures in the frame (ignoring time
        # lag 0, we can see that the next major peak is the lag where we see our
        # fundamental frequency)
        windowed = frame * np.hanning(frame_len)

        # For some reason, this fn only calls np.correlate when making the
        # autocorr, but theres some kind of issue where we end up with invalid
        # values after using that matrix.
        #estimated_f0, estimated_lag, pos_lags, pos_autocorr, lags, autocorr = \
        #    estimate_f0_robust(windowed, fs) # min hz 20, max 1000
        #autocorr = autocorr[frame_len-1:frame_len+lpc_order]

        autocorr = np.correlate(windowed, windowed, mode='full')[frame_len-1:frame_len+lpc_order]

        autocorr[0] *= 1.0001  # tiny regularisation to keep the Toeplitz system well-conditioned
        b = np.zeros(lpc_order+1); b[0] = 1.0
        a = solve_toeplitz(autocorr, b)
        a = a / a[0]  # A(z) = 1 + a_1 z^-1 + ... + a_p z^-p

        w_num, w_den = get_speex_weighting_coeffs(a, 0.9, 0.6)

        if lpc_quantization:
            quantized_indices, a, min_val, max_val = quantize_lpc(a, lpc_bits)

        for s in range(subframes_per_frame):
            start = f*frame_len + s*subframe_len
            target = data[start:start+subframe_len]

            # LPC residual of this subframe, using the true past samples as filter memory
            ctx = data[max(0, start-lpc_order):start]
            ctx = np.concatenate((np.zeros(lpc_order-len(ctx)), ctx))
            x = lfilter(a, [1.0], np.concatenate((ctx, target)))[lpc_order:]

            # --- long-term (pitch) prediction: x_F0^*, argmax over candidate lags T ---
            # from our residual x we find the candidate in the subframe which we
            # like
            scores = []
            w_x = lfilter(w_num, w_den, x)
            for T in range(lag_min, lag_max+1):
                cand = excitation_history[-T:len(excitation_history)-T+subframe_len]
                # introduce weighting filter during lag selection, this is
                # adjusting to eq 5 in the aalto book.
                # I'm gonna stick with the simplest case noise codebook (one
                # signed pulse) and shove it through a pink noise filter.
                #
                # applying the weighting filter during the actual syntheisis
                # ended up actually breaking my signal, it was mostly high
                # amplitude junk, and i had to implement a heavy attenuation on
                # my final decode summation. this seems like a more reasonable
                # place to put it since this is where the subframe analysis is
                w_cand = lfilter(w_num, w_den, cand)
                scores.append(np.dot(w_x, w_cand)**2 / (np.dot(w_cand, w_cand) + 1e-9))
            # pick the best time lag and slice that data, estimate f0
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

            # Throwing a pink filter on it, cause I want to spread out the
            # energy in the noise and not just have a delta at my subframe
            # This could stem from my own misunderstanding but i like the
            # results im getting with this
            if apply_color_filter:
                color_filter = ColoredNoiseFilter(spectral_slope=1.0)
                x_noise = color_filter.apply(x_noise, sr=fs, attenuation=1)

            gain_noise = x_prime[pulse_pos] * pulse_sign                          # gamma_noise^*
            gain_noise = noise_gain_levels[np.argmin(np.abs(noise_gain_levels - gain_noise))]

            # --- quantized excitation and reconstruction, x_hat^* ---
            excitation = (gain_f0*x_f0 + gain_noise*x_noise)
            excitation_history = np.concatenate((excitation_history[subframe_len:], excitation))

            # inverse filtering
            synth_out, synth_state = lfilter([1.0], a, excitation, zi=synth_state)
            decoded[start:start+subframe_len] = synth_out

            bits_used += bits_per_subframe

    decoded = np.clip(decoded, -1.0, 1.0)
    # forces an output for low lpc bitrates
    if np.isnan(decoded).any:
        decoded = np.nan_to_num(decoded)

    duration_s = len(data)/fs
    bitrate_kbps = bits_used/duration_s/1000
    print(f"Encoded {duration_s:.2f}s of speech at {bitrate_kbps:.1f} kbit/s ")

    if not no_graphs:
        data_f = data.astype(np.float32)
        decoded_f = decoded.astype(np.float32)

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(8, 4), sharex=True)

        # transients
        # original
        librosa.display.waveshow(data_f, sr=fs, ax=ax1)
        ax1.set_title("Original speech sample")
        ax1.set_ylabel("Amplitude")

        # decoded
        librosa.display.waveshow(decoded_f, sr=fs, ax=ax2)
        ax2.set_title(f"CELP-decoded speech sample ({bitrate_kbps:.1f} kbit/s)")
        ax2.set_xlabel("Time (s)")
        ax2.set_ylabel("Amplitude")

        plt.tight_layout()
        plt.show()

        ipd.display(ipd.Audio(data_f, rate=fs))
        ipd.display(ipd.Audio(decoded_f, rate=fs))

        # spectrograms

        S_orig = librosa.stft(data_f)
        S_orig_db = librosa.amplitude_to_db(np.abs(S_orig), ref=np.max)

        S_dec = librosa.stft(decoded_f)
        S_dec_db = librosa.amplitude_to_db(np.abs(S_dec), ref=np.max)

        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8), sharex=True)

        # origina
        img1 = librosa.display.specshow(S_orig_db, sr=fs, x_axis='time', y_axis='hz', ax=ax1)
        ax1.set_title("Original Speech Spectrogram (STFT)")
        fig.colorbar(img1, ax=ax1, format="%db")

        # decoded
        img2 = librosa.display.specshow(S_dec_db, sr=fs, x_axis='time', y_axis='hz', ax=ax2)
        ax2.set_title(f"CELP-decoded Spectrogram ({bitrate_kbps:.1f} kbit/s)")
        fig.colorbar(img2, ax=ax2, format="%db")

        plt.tight_layout()
        plt.show()

        # Display Audio players
        ipd.display(ipd.Audio(data_f, rate=fs))
        ipd.display(ipd.Audio(decoded_f, rate=fs))

    wavfile.write("output.wav", fs, decoded)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Robust Autocorrelation F0 estimator.")
    parser.add_argument("wav_file", nargs="?", help="Path to the .wav file.")
    parser.add_argument("--colored", action='store_true')
    parser.add_argument("--lpc_quantization", action='store_true')
    parser.add_argument("--no_graphs", action='store_true')

    args = parser.parse_args()

    main(args.wav_file, args.colored, args.lpc_quantization, args.no_graphs)
