;; Base oscillator
(define osc (oscillator (note -24)))

(define fenv (frequency (envelope (set 0.6) (exp 70ms -0.2))))

;; Simple filter sweep.
(highPass
 30Hz
 (saturate
  (* (lowPass4
      (mix
       -9dB (sawtooth (overtone 2 osc))
       -6dB (sawtooth osc))
      fenv 1.5)
     (envelope (lin 0.05ms 1) (exp 100ms 0.2) (delay 600ms)
	       (gate) (exp 50ms 0) (stop)))))
