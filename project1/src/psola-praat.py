from parselmouth.praat import call
import parselmouth

def change_pitch(sound, factor):
    # taken from parselmouth examples
    manipulation = call(sound, "To Manipulation", 0.01, 75, 600)

    pitch_tier = call(manipulation, "Extract pitch tier")

    call(pitch_tier, "Multiply frequencies", sound.xmin, sound.xmax, factor)

    call([pitch_tier, manipulation], "Replace pitch tier")
    return call(manipulation, "Get resynthesis (overlap-add)")

def change_duration(sound, factor):
    duration = sound.get_total_duration()

    manipulation = call(sound, "To Manipulation", 0.01, 75, 600)

    duration_tier = call(manipulation, "Extract duration tier")

    # Add duration points to stretch time (e.g., factor of 2.0 = twice as long)
    call(duration_tier, "Add point", 0.0, factor)
    call(duration_tier, "Add point", duration, factor)

    call([duration_tier, manipulation], "Replace duration tier")

    return call(manipulation, "Get resynthesis (overlap-add)")

if __name__ == "__main__":
    import sys

    if len(sys.argv) != 4:
        print(f"Usage: {sys.argv[0]} [stretch | pitch] [path to wav] [stretch factor]")
        exit(1)

    try:
        option = sys.argv[1]
        input_wav = sys.argv[2]
        input_wav_name = input_wav.removesuffix(".wav")
        factor = float(sys.argv[3])

        audio = parselmouth.Sound(input_wav)

        match option:
            case "stretch":
                modified = change_duration(audio, factor)
            case "pitch":
                modified = change_pitch(audio, factor)

        modified.save(f"{input_wav_name}_{option}_{factor}_psola.wav", "WAV")
    except Exception as e:
        print(f"An error occurred: {e}")

