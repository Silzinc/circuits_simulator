import matplotlib.pyplot as plt
import numpy as np
import os
from sys import argv

os.chdir(os.path.dirname(os.path.abspath(__file__)))

# Load data
with open("tensions1.txt", 'r') as tensions1_file:
  tensions1 = [float(s) for s in tensions1_file.read()[1:-1].split(', ')]
with open("tensions2.txt", 'r') as tensions2_file:
  tensions2 = [float(s) for s in tensions2_file.read()[1:-1].split(', ')]

assert len(tensions1) == len(tensions2)

# Parameters
duration = float(argv[1]) # Duration of the simulation passed by Rust
t = np.linspace(0, duration, len(tensions1))

# Plot
fig = plt.figure()
fig.suptitle("Tensions over time in a RLC circuit")
ax1, ax2 = fig.subplots(2, 1)

ax1.plot(t, tensions1)
ax1.set_ylabel("Tension on the source (V)")

ax2.plot(t, tensions2)
ax2.set_ylabel("Tension on the capacitor (V)")

plt.xlabel("Time (s)")
plt.show()