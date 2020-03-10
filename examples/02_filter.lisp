;; Sawtooth filter sweep.
(highPass
 30Hz
 (lowPass2
  (sawtooth (oscillator (note 0)))
  (frequency (envelope (set 1.0) (lin 2s -1.0) (stop)))
  2.0))
