;; Base oscillator
(define osc (oscillator (note 0)))

(define fenv (frequency (envelope (exp 100ms 0.5) (exp 150ms 0.1))))

;; Simple filter sweep.
(highPass
 150Hz
 (saturate
  (* (lowPass2
      (sawtooth osc)
      fenv 2)
     (envelope (exp 20ms 1) (gate) (exp 80ms 0) (stop)))))
