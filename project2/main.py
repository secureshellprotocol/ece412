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

from noise_filter import ColoredNoiseFilter
from utils.autocorr import estimate_f0_robust
from utils.quantize_lpc import quantize_lpc

## all of my base case extenstions will be introduced as functions here

def print_help():
    print(f'Not enough arguments provided!')
    print(f'usage: {sys.argv[0]} input.wav output.wav')

import numpy as np

def get_speex_weighting_coeffs(a, g1, g2):
    """
    Returns the numerator and denominator coefficients for the
    Speex weighting filter W(z) = A(z/g1) / A(z/g2).

    Args:
        a (np.array): The LPC coefficients [a1, a2, ..., ap].
        g1 (float): The constant gamma1 (e.g., 0.9).
        g2 (float): The constant gamma2 (e.g., 0.6).

    Returns:
        num_coeffs (np.array): Numerator coefficients [1, b1, b2, ...].
        den_coeffs (np.array): Denominator coefficients [1, d1, d2, ...].
    """
    # Create an array of powers [1, 2, ..., p] to match the coefficient index
    p = len(a)
    powers = np.arange(1, p + 1)

    # Compute b_k = a_k * g1^k and d_k = a_k * g2^k
    # We prepend 1.0 because the polynomial starts with 1 + ...
    num_coeffs = np.concatenate(([1.0], a * (g1 ** powers)))
    den_coeffs = np.concatenate(([1.0], a * (g2 ** powers)))

    return num_coeffs, den_coeffs

def weighted_norm(x, x_hat, W):
    x_diff = x - x_hat
    return np.transpose(x_diff) @ np.transpose(W) @ W @ x_diff


def main():
    if len(sys.argv) != 3:
        print_help()
        exit(1)
    filename = sys.argv[1]
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
    # todo figure out where the bits numbers came from
    bits_used = 0
    lpc_bits = 16
    bits_per_subframe = 8 + 5 + 7 + 1 + 5 + lpc_bits
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
        estimated_f0, estimated_lag, pos_lags, pos_autocorr, _, _ = \
            estimate_f0_robust(windowed, fs) # min hz 20, max 1000

        autocorr = np.correlate(windowed, windowed, mode='full')[frame_len-1:frame_len+lpc_order]
        autocorr[0] *= 1.0001  # tiny regularisation to keep the Toeplitz system well-conditioned
        b = np.zeros(lpc_order+1); b[0] = 1.0
        a = solve_toeplitz(autocorr, b)
        a = a / a[0]  # A(z) = 1 + a_1 z^-1 + ... + a_p z^-p

        w_num, w_den = get_speex_weighting_coeffs(a, 0.9, 0.6)

        quantized_indices, a, min_val, max_val = quantize_lpc(a, lpc_bits)



        for s in range(subframes_per_frame):
            start = f*frame_len + s*subframe_len
            target = data[start:start+subframe_len]

            # LPC residual of this subframe, using the true past samples as filter memory
            ctx = data[max(0, start-lpc_order):start]
            ctx = np.concatenate((np.zeros(lpc_order-len(ctx)), ctx))
            x = lfilter(a, [1.0], np.concatenate((ctx, target)))[lpc_order:]

            # --- long-term (pitch) prediction: x_F0^*, argmax over candidate lags T ---
            scores = []
            w_x = lfilter(w_num, w_den, x)
            for T in range(lag_min, lag_max+1):
                cand = excitation_history[-T:len(excitation_history)-T+subframe_len]
                # introduce weighting filter during lag selection, this is
                # adjusting to eq 5 in the aalto book
                # I'm gonna stick with the simplest case noise codebook (one
                # signed pulse) and shove it through a pink noise filter
                w_cand = lfilter(w_num, w_den, cand)
                scores.append(np.dot(w_x, w_cand)**2 / (np.dot(w_cand, w_cand) + 1e-9))
            # pick the best time lag and slice that data
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

            # Throwing a color filter on it, cause I want to spread out the
            # energy in the noise and not just have a delta at my subframe
            # This could stem from my own misunderstanding but i like the
            # results im getting with this
            color_filter = ColoredNoiseFilter(spectral_slope=0.5)
            x_noise = color_filter.apply(x_noise, sr=fs, attenuation=0.707)

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
    decoded = np.nan_to_num(decoded)

    duration_s = len(data)/fs
    bitrate_kbps = bits_used/duration_s/1000
    print(f"Encoded {duration_s:.2f}s of speech at {bitrate_kbps:.1f} kbit/s ")

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

    wavfile.write(sys.argv[2], fs, decoded)



if __name__ == "__main__":
    main()
