import parselmouth
from parselmouth.praat import call

# 1. Load your sound
sound = parselmouth.Sound("television.wav")
duration = sound.get_total_duration()

# 2. Create a Manipulation object (time step, min pitch, max pitch)
manipulation = call(sound, "To Manipulation", 0.01, 75, 600)

# 3. Extract the Duration Tier
duration_tier = call(manipulation, "Extract duration tier")

# 4. Add duration points to stretch time (e.g., factor of 2.0 = twice as long)
# Format: Add point at (time, stretch_factor)
factor = 2.0
call(duration_tier, "Add point", 0.0, factor)
call(duration_tier, "Add point", duration, factor)

# 5. Replace the duration tier back into the manipulation object
call([duration_tier, manipulation], "Replace duration tier")

# 6. Resynthesize the sound using overlap-add
time_stretched_sound = call(manipulation, "Get resynthesis (overlap-add)")

time_stretched_sound.save("balls.wav", "WAV")
