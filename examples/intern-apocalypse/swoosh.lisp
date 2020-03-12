;; Sword swoosh noise.
(* (lowPass2
    (noise)
    (frequency (envelope (set 0.0) (lin 150ms 0.7) (lin 150ms 0.0)))
    2)
   (envelope (set 0.0) (lin 150ms 0.5) (lin 150ms 0.0) (stop)))
