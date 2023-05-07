(setdyn :syspath "./target/debug")
(use libjsyn)

(var handle (play (sine-hz 220)))
(os/sleep 2)
