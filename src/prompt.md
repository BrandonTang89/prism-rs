Help me to incorporate testing of the brp model with brp.prop

Attached is the expected output from running the brp example with the PRISM library.
Remember to specify 'N=2,MAX=3' as the constants. 

Fix any bugs you may find along the way and ensure all tests still pass


direnv loaded/allowed prism-games ❯ prism/bin/prism prism-examples/dtmcs/brp/brp.pm -const N=2,MAX=3 -mtbdd prism-examples/dtmcs/brp/brp.pctl
WARNING: A restricted method in java.lang.System has been called
WARNING: java.lang.System::loadLibrary has been called by prism.PrismNative in an unnamed module (file:/home/brandon/playspace/prism-games/prism/classes/)
WARNING: Use --enable-native-access=ALL-UNNAMED to avoid a warning for callers in this module
WARNING: Restricted methods will be blocked in a future release unless native access is enabled

PRISM-games
===========

Version: 3.2.1 (based on PRISM 4.8.1.dev)
Date: Wed Apr 15 11:52:19 BST 2026
*** CUSTOM BUILD - CHANGES ACTIVE ***
Hostname: nixos
Memory limits: cudd=1g, java(heap)=1g
Command line: prism-games prism-examples/dtmcs/brp/brp.pm -const 'N=2,MAX=3' -mtbdd prism-examples/dtmcs/brp/brp.pctl

Parsing PRISM model file "prism-examples/dtmcs/brp/brp.pm"...

Type:        DTMC
Modules:     sender receiver checker channelK channelL
Variables:   s srep nrtr i bs s_ab fs ls r rrep fr lr br r_ab recv T k l
Rewards:     1

Parsing properties file "prism-examples/dtmcs/brp/brp.pctl"...

7 properties:
(1) P=? [ true U srep=1&rrep=3&recv ]
(2) P=? [ true U srep=3&!(rrep=3)&recv ]
(3) P=? [ true U s=5 ]
(4) P=? [ true U s=5&srep=2 ]
(5) P=? [ true U s=5&srep=1&i>8 ]
(6) P=? [ true U !(srep=0)&!recv ]
(7) R=? [ F "deadlock" ]

---------------------------------------------------------------------

Model checking: P=? [ true U srep=1&rrep=3&recv ]
Model constants: N=2,MAX=3

Building model (engine:symbolic)...
Model constants: N=2,MAX=3

Translating modules to MTBDD...

Computing reachable states...

Reachability (BFS): 23 iterations in 0.01 seconds (average 0.000435, setup 0.00)

Time for model construction: 0.016 seconds.

Warning: Deadlocks detected and fixed in 8 states

Type:        DTMC
States:      116 (1 initial)
Transitions: 147

Transition matrix: 1119 nodes (6 terminal), 147 minterms, vars: 29r/29c

yes = 0, no = 116, maybe = 0

Value in the initial state: 0.0

Time for model checking: 0.001 seconds.

Result: 0.0 (exact floating point)

---------------------------------------------------------------------

Model checking: P=? [ true U srep=3&!(rrep=3)&recv ]
Model constants: N=2,MAX=3

yes = 0, no = 116, maybe = 0

Value in the initial state: 0.0

Time for model checking: 0.0 seconds.

Result: 0.0 (exact floating point)

---------------------------------------------------------------------

Model checking: P=? [ true U s=5 ]
Model constants: N=2,MAX=3

Prob0: 13 iterations in 0.00 seconds (average 0.000000, setup 0.00)

Prob1: 13 iterations in 0.00 seconds (average 0.000000, setup 0.00)

yes = 14, no = 20, maybe = 82

Computing remaining probabilities...
Engine: MTBDD

Iteration matrix MTBDD... [nodes=787] [15.4 Kb]
Diagonals MTBDD... [nodes=305] [6.0 Kb]

Starting iterations...

Jacobi: 33 iterations in 0.01 seconds (average 0.000303, setup 0.00)

Value in the initial state: 1.5772293325415632E-6

Time for model checking: 0.011 seconds.

Result: 1.5772293325415632E-6 (+/- 1.2032218988391367E-12 estimated; rel err 7.628706073455107E-7)

---------------------------------------------------------------------

Model checking: P=? [ true U s=5&srep=2 ]
Model constants: N=2,MAX=3

Prob0: 18 iterations in 0.00 seconds (average 0.000000, setup 0.00)

Prob1: 11 iterations in 0.00 seconds (average 0.000000, setup 0.00)

yes = 7, no = 27, maybe = 82

Computing remaining probabilities...
Engine: MTBDD

Iteration matrix MTBDD... [nodes=787] [15.4 Kb]
Diagonals MTBDD... [nodes=305] [6.0 Kb]

Starting iterations...

Jacobi: 33 iterations in 0.01 seconds (average 0.000303, setup 0.00)

Value in the initial state: 7.886142909415634E-7

Time for model checking: 0.008 seconds.

Result: 7.886142909415634E-7 (+/- 1.2032218070487946E-12 estimated; rel err 1.525741824450343E-6)

---------------------------------------------------------------------

Model checking: P=? [ true U s=5&srep=1&i>8 ]
Model constants: N=2,MAX=3

yes = 0, no = 116, maybe = 0

Value in the initial state: 0.0

Time for model checking: 0.0 seconds.

Result: 0.0 (exact floating point)

---------------------------------------------------------------------

Model checking: P=? [ true U !(srep=0)&!recv ]
Model constants: N=2,MAX=3

Prob0: 11 iterations in 0.00 seconds (average 0.000000, setup 0.00)

Prob1: 3 iterations in 0.00 seconds (average 0.000000, setup 0.00)

yes = 5, no = 103, maybe = 8

Computing remaining probabilities...
Engine: MTBDD

Iteration matrix MTBDD... [nodes=194] [3.8 Kb]
Diagonals MTBDD... [nodes=305] [6.0 Kb]

Starting iterations...

Jacobi: 9 iterations in 0.00 seconds (average 0.000000, setup 0.00)

Value in the initial state: 1.6000000000000003E-7

Time for model checking: 0.004 seconds.

Result: 1.6000000000000003E-7 (exact floating point)
