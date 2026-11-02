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

