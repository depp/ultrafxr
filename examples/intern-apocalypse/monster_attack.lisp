;; (highPass
;;  50Hz
;;  (* (lowPass4 (noise)
;; 	      (frequency (envelope (set 0.6) (lin 200ms -0.3)))
;; 	      0.7)
;;     (envelope (lin 20ms 1) (lin 300ms 0) (stop))))

(* (sawtooth (phase-mod
	      (frequency (envelope (set -0.2) (lin 400ms -0.9)))
	      -4dB (* (noise)
		     (envelope (set 1) (exp 200ms 0)))))
   (envelope (lin 20ms 1) (exp 100ms 0) (stop)))

