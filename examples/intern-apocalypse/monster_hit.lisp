(highPass
 100Hz
 (*
  (sine
   (phase-mod
    (frequency (envelope (set -0.2) (lin 200ms -0.9)))
    0.5 (rectify
         (lowPass2
          (noise)
          (frequency (envelope (set 0.4) (lin 200ms -0.5)))
          3.0))))
  (envelope (lin 1ms 1) (lin 300ms 0) (stop))))
