(highPass
 80Hz
 (*
  (saturate
   (lowPass2
    (sawtooth
     (oscillator
      (frequency (envelope (set -0.3) (exp 500ms -0.1)))))
    (frequency (envelope (set 0.5) (lin 600ms 0)))
    4.0))
  (envelope (exp 30ms 1.0) (delay 300ms) (lin 400ms 0) (stop))))
