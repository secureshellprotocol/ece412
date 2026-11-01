# generated using chatterbox.ee
# this is more of a look at how we are finding f0
# main idea is voiced audio will correlate to some kind of f0, unvoiced audio
# wont

import numpy as np
import matplotlib.pyplot as plt
import argparse
import sys
from scipy.io import wavfile
from scipy.signal import find_peaks

def parabolic_interpolation(y_values, peak_idx):
    if 0 < peak_idx < len(y_values) - 1:
        alpha = y_values[peak_idx - 1]
        beta = y_values[peak_idx]
        gamma = y_values[peak_idx + 1]
        # Avoid division by zero
        denom = (alpha - 2 * beta + gamma)
        if abs(denom) < 1e-10:
            return float(peak_idx)
        offset = 0.5 * (alpha - gamma) / denom
        return peak_idx + offset
    return float(peak_idx)

def estimate_f0_robust(signal, fs, min_f0=20.0, max_f0=1000.0):
    """
    Improved F0 estimation: Searches for the HIGHEST peak within
    a specified frequency range rather than the FIRST peak.
    """
    N = len(signal)
    autocorr = np.correlate(signal, signal, mode='full')
    lags = np.arange(-N + 1, N)
    mid_idx = N - 1

    # Focus on positive lags
    positive_lags = lags[mid_idx + 1:]
    positive_corr = autocorr[mid_idx + 1:]

    # Normalize
    if np.max(positive_corr) != 0:
        positive_corr /= np.max(positive_corr)

    # 1. Find all potential peaks
    # We use a smaller prominence to ensure we don't miss the true peak
    peaks, _ = find_peaks(positive_corr, prominence=0.05, distance=5)

    if len(peaks) == 0:
        return 0, 0, positive_lags, positive_corr, lags, autocorr

    # 2. Filter peaks by frequency range
    # Convert min_f0/max_f0 to lag limits (in samples/seconds)
    # Period T = 1/f -> Lag_seconds = 1/f
    # Lag_samples = (1/f) * fs
    min_lag_s = 1.0 / max_f0
    max_lag_s = 1.0 / min_f0

    valid_peaks = []
    for p in peaks:
        lag_s = positive_lags[p] / fs
        if min_lag_s <= lag_s <= max_lag_s:
            valid_peaks.append(p)

    if not valid_peaks:
        # Fallback: if no peaks in range, just return the first peak found
        # (though this indicates the range settings are likely wrong)
        best_peak_idx = peaks[0]
    else:
        # 3. STRATEGY CHANGE: Pick the peak with the HIGHEST magnitude
        # This is much more robust against local noise/harmonics
        best_peak_idx = valid_peaks[np.argmax(positive_corr[valid_peaks])]

    # 4. Parabolic Interpolation for sub-sample precision
    refined_idx = parabolic_interpolation(positive_corr, best_peak_idx)

    # Map the fractional index back to the lag value
    step = positive_lags[1] - positive_lags[0]
    refined_lag_val = positive_lags[int(best_peak_idx)] + (refined_idx - best_peak_idx) * step

    if refined_lag_val <= 0:
        return 0, 0, positive_lags, positive_corr, lags, autocorr

    estimated_f0 = fs / refined_lag_val
    return estimated_f0, refined_lag_val, positive_lags, positive_corr, lags, autocorr

def load_wav(filename, offset, duration):
    try:
        fs, data = wavfile.read(filename)
        if data.dtype.kind in 'iu':
            data = data.astype(np.float64) / np.iinfo(data.dtype).max
        if len(data.shape) > 1:
            data = data.mean(axis=1)
        start_sample = int(offset * fs)
        end_sample = int((offset + duration) * fs)
        if start_sample >= len(data):
            raise ValueError(f"Offset {offset}s is beyond the end of the file.")
        signal = data[start_sample:end_sample]
        if len(signal) == 0:
            raise ValueError("The resulting slice is empty.")
        return signal, fs
    except Exception as e:
        print(f"Error loading WAV: {e}")
        sys.exit(1)

def main():
    parser = argparse.ArgumentParser(description="Robust Autocorrelation F0 estimator.")
    parser.add_argument("wav_file", nargs="?", help="Path to the .wav file.")
    parser.add_argument("--offset", type=float, default=0.0, help="Start offset [s]")
    parser.add_argument("--duration", type=float, default=1.0, help="Duration [s]")
    parser.add_argument("--min_f0", type=float, default=20.0, help="Minimum frequency to search [Hz]")
    parser.add_argument("--max_f0", type=float, default=1000.0, help="Maximum frequency to search [Hz]")
    parser.add_argument("--freq", type=float, default=5.0, help="Target freq (sine only)")
    parser.add_argument("--fs", type=float, default=1000.0, help="Sampling rate (sine only)")

    args = parser.parse_args()

    if args.wav_file:
        signal, fs = load_wav(args.wav_file, args.offset, args.duration)
    else:
        fs = args.fs
        t = np.arange(0, args.duration, 1/fs)
        signal = np.sin(2 * np.pi * args.freq * t)

    # Pass frequency constraints to the robust estimator
    f0_est, lag_val, pos_lags, pos_corr, lags, autocorr = \
        estimate_f0_robust(signal, fs, args.min_f0, args.max_f0)

    print("-" * 30)
    print(f"Search Range: {args.min_f0}Hz - {args.max_f0}Hz")
    print(f"Result:       {f0_est:.4f} Hz")
    print(f"Lag:          {lag_val/fs:.6f} s")
    print("-" * 30)

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))
    plt.subplots_adjust(hspace=0.4)

    t_axis = np.linspace(0, len(signal)/fs, len(signal))
    ax1.plot(t_axis, signal, color='blue', linewidth=0.5)
    ax1.set_title(f"Signal Segment")
    ax1.grid(True)

    ax2.plot(pos_lags / fs, pos_corr, color='red', label='Autocorrelation')
    if f0_est > 0:
        ax2.axvline(x=lag_val/fs, color='green', linestyle='--',
                    label=f'Highest Peak in Range ({lag_val/fs:.4f}s)')
    ax2.set_title(f"Autocorrelation (Looking for highest peak between {args.min_f0}-{args.max_f0}Hz)")
    ax2.set_xlabel("Lag [s]")
    ax2.set_ylabel("Normalized Magnitude")
    ax2.legend()
    ax2.grid(True)

    plt.show()

if __name__ == "__main__":
    main()
