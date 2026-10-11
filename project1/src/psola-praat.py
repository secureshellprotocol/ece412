from parselmouth.praat import call
import parselmouth

def change_pitch(sound, factor):
    manipulation = call(sound, "To Manipulation", 0.01, 75, 600)

    pitch_tier = call(manipulation, "Extract pitch tier")

    call(pitch_tier, "Multiply frequencies", sound.xmin, sound.xmax, factor)

    call([pitch_tier, manipulation], "Replace pitch tier")
    return call(manipulation, "Get resynthesis (overlap-add)")

# --- Execution ---
if __name__ == "__main__":
    # Ensure 'test.wav' is in your directory!
    input_wav = "test.wav"

    audio = parselmouth.Sound(input_wav)

    try:
        # Test Pitch Shift (Higher)
        pitch_shifted_audio = change_pitch(audio, 2.0)
        pitch_shifted_audio.save("output.wav", "WAV")

        # Test Time Stretch (Faster)
        #time_stretch_psola(input_wav, "fast.wav", 0.7)

    except Exception as e:
        print(f"An error occurred: {e}")
