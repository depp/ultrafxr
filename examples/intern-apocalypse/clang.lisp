(define mod2
  (* (sine 9120Hz)
     (envelope (set 1.0) (lin 100ms 0.9) (set 0.0))))

(define mod1
  (* (sine
      (phase-mod
       3000Hz
       0dB mod2))
     (envelope (set 1.0) (lin 500ms 0.0))))

(* (sine
    (phase-mod
     1200Hz
     0dB mod1))
   (envelope (set 1.0) (lin 600ms 0.0) (stop)))
