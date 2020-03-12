;; Base oscillators.
(define osc (oscillator (note 0)))
(define osc2 (overtone 2 osc))
(define osc3 (overtone 3 osc))

(define env1 (envelope (exp 0.5ms 1) (exp 400ms 0)))
(define env2 (envelope (exp 20ms 1) (exp 800ms 0) (stop)))

;; Modulators.
(define mod1 (* (sine osc3)
		env1))
(define mod2 (* (sine (phase-mod osc3
				 -12dB mod1))
		env1))
(define mod3 (* (sine osc2)
		env2))

;; Output.
(highPass
 40Hz
 (mix
  -6dB (* (sine (phase-mod osc
			   -16dB mod2))
	  env1)
  -6dB (* (sine (phase-mod osc
			   -12dB mod3))
	  env2)))
