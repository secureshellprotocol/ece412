# generated using chatterbox.ee
import numpy as np

def quantize_lpc(coeffs, bits):
    """
    Quantizes a vector of coefficients to a specified bit precision.

    Args:
        coeffs (np.array): The input LPC coefficients.
        bits (int): The number of bits per coefficient.

    Returns:
        quantized_indices (np.array): The integer representations (0 to 2^bits - 1).
        reconstructed (np.array): The coefficients after dequantization.
        min_val (float): The minimum value used for scaling.
        max_val (float): The maximum value used for scaling.


    this code basically just acts like how a physical ladder adc would, some
    analog value gets snapped to some fixed level of bits

    ladder adc
     - r + - r  +
        |       |       ...
        2r      2r

    this wonderful graphic was not ai generated
    """
    # 1. Determine the range of the coefficients
    min_val = np.min(coeffs)
    max_val = np.max(coeffs)

    # Handle the case where all coefficients are the same
    if max_val == min_val:
        return np.zeros_like(coeffs, dtype=np.int32), coeffs, min_val, max_val

    # 2. Calculate the number of discrete levels
    levels = 2**bits

    # 3. Map the coefficients to the range [0, levels - 1]
    # We use (levels - 1) so that the max value maps exactly to the last index
    scaled = (coeffs - min_val) / (max_val - min_val) * (levels - 1)

    # 4. Round to the nearest integer and clip to ensure we stay in bounds
    quantized_indices = np.round(scaled).astype(np.int32)
    quantized_indices = np.clip(quantized_indices, 0, levels - 1)

    # 5. Reconstruction (Dequantization)
    # This simulates what the decoder does
    reconstructed = (quantized_indices / (levels - 1)) * (max_val - min_val) + min_val

    return quantized_indices, reconstructed, min_val, max_val

# --- Example Usage ---
if __name__ == "__main__":
    # Simulated LPC coefficients (e.g., 10th order)
    lpc_coeffs = np.array([0.12, -0.45, 0.88, -0.23, 0.05, -0.01, 0.34, -0.67, 0.11, -0.09])

    # Quantize to 4 bits (16 levels)
    bits = 4
    indices, recon, v_min, v_max = quantize_lpc(lpc_coeffs, bits)

    print(f"Original:      {lpc_coeffs}")
    print(f"Quantized (int): {indices}")
    print(f"Reconstructed: {recon}")
    print(f"Quantization Error (MSE): {np.mean((lpc_coeffs - recon)**2):.6f}")
