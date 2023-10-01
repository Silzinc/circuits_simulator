import matplotlib.pyplot as plt
import numpy as np
import os
from sys import argv

os.chdir(os.path.dirname(os.path.abspath(__file__)))

# Load data
with open("currents.txt", 'r') as currents_file:
  currents = [float(s) for s in currents_file.read()[1:-1].split(', ')]
with open("tensions.txt", 'r') as tensions_file:
  tensions = [float(s) for s in tensions_file.read()[1:-1].split(', ')]

assert len(currents) == len(tensions)

# Parameters
duration = float(argv[1])
t = np.linspace(0, duration, len(currents))

# Plot
fig = plt.figure()
fig.suptitle("Currents and tensions over time")
ax1, ax2 = fig.subplots(2, 1)

ax1.plot(t, currents)
ax1.set_ylabel("Current (A)")

ax2.plot(t, tensions)
ax2.set_ylabel("Tension (V)")

plt.xlabel("Time (s)")
plt.show()