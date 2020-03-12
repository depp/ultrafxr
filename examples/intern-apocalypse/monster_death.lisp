(highPass
 60Hz
 (*
  (sawtooth
   (phase-mod
    (oscillator (frequency (envelope (set -0.1) (lin 200ms -0.6) (lin 1000ms -0.4))))
    2.0 (lowPass2
         (noise)
         (frequency (envelope (set -0.5) (lin 400ms -0.2)))
         1.9)))
 (envelope (lin 10ms 1) (delay 300ms) (lin 1000ms 0) (stop))))
