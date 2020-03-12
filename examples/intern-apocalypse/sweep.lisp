;; Base oscillator
(define osc (oscillator (note -12)))

;; Simple filter sweep.
(saturate
 (* (lowPass2
     (mix
      -6dB (sawtooth (overtone 2 osc))
      -6dB (sawtooth (overtone 3 osc)))
     (frequency (envelope (set 1) (lin 1s -1)))
     5.0)
    (envelope (lin 100ms 1) (delay 800ms) (lin 100ms 0) (stop))))
