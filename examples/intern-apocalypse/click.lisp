;; Base oscillators.
(define osc (oscillator (note 0)))
(define osc2 (overtone 5 osc))

(define mod1
  (* (sine osc2)
     (envelope (set 1.0) (exp 10ms 0.0))))

(* (sine (phase-mod
	  osc
	  -12dB mod1))
   (envelope (lin 2ms 1.0) (exp 90ms 0) (stop)))
