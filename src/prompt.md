Hmm for simple2_dtmc.prism, somehow the number of nodes in my transition DD is different
from that of PRISM's. Can you help me check if this is a real bug or if its oka, maybe its just due to different variable ordering or encoding


prism-games master*
direnv loaded/allowed prism-games ❯ prism/bin/prism prism/etc/tests/dtmc_pctl.prism  -mtbdd -exportdot tmp.dot
WARNING: A restricted method in java.lang.System has been called
WARNING: java.lang.System::loadLibrary has been called by prism.PrismNative in an unnamed module (file:/home/brandon/playspace/prism-games/prism/classes/)
WARNING: Use --enable-native-access=ALL-UNNAMED to avoid a warning for callers in this module
WARNING: Restricted methods will be blocked in a future release unless native access is enabled

PRISM-games
===========

Version: 3.2.1 (based on PRISM 4.8.1.dev)
Date: Mon Apr 13 11:12:25 BST 2026
*** CUSTOM BUILD - CHANGES ACTIVE ***
Hostname: nixos
Memory limits: cudd=1g, java(heap)=1g
Command line: prism-games prism/etc/tests/dtmc_pctl.prism -mtbdd -exportdot tmp.dot

Parsing PRISM model file "prism/etc/tests/dtmc_pctl.prism"...

Type:        DTMC
Modules:     M
Variables:   s
Rewards:     "time" "r"

Building model (engine:symbolic)...

Translating modules to MTBDD...

Computing reachable states...

Reachability (BFS): 4 iterations in 0.00 seconds (average 0.000000, setup 0.00)

Time for model construction: 0.009 seconds.

Type:        DTMC
States:      6 (1 initial)
Transitions: 9

Transition matrix: 29 nodes (4 terminal), 9 minterms, vars: 3r/3c

Exporting model in DD Dot format to file "tmp.dot"...


prism-games master*
direnv loaded/allowed prism-games ❯ cat tmp.dot
digraph "DD" {
size = "7.5,10"
center = true;
edge [dir = none];
{ node [shape = plaintext];
  edge [style = invis];
  "CONST NODES" [style = invis];
" s.0 " -> " s'.0 " -> " s.1 " -> " s'.1 " -> " s.2 " -> " s'.2 " -> "CONST NODES";
}
{ rank = same; node [shape = box]; edge [style = invis];
"  DD  "; }
{ rank = same; " s.0 ";
"0x395";
}
{ rank = same; " s'.0 ";
"0x37d";
"0x394";
}
{ rank = same; " s.1 ";
"0x373";
"0x288";
"0x32d";
"0x393";
}
{ rank = same; " s'.1 ";
"0x36a";
"0x287";
"0x32c";
"0x345";
"0x2b5";
"0x2f8";
}
{ rank = same; " s.2 ";
"0x368";
"0x332";
"0x293";
"0x286";
"0x369";
"0x306";
"0x2f7";
}
{ rank = same; " s'.2 ";
"0x2fd";
"0x276";
"0x331";
"0x31e";
"0x285";
}
{ rank = same; "CONST NODES";
{ node [shape = box]; "0x2fc";
"0x23f";
"0x330";
}
}
"  DD  " -> "0x395" [style = solid];
"0x395" [label = ""];
"0x395" -> "0x37d";
"0x395" -> "0x394" [style = dotted];
"0x37d" [label = ""];
"0x37d" -> "0x288";
"0x37d" -> "0x32d" [style = dotted];
"0x394" [label = ""];
"0x394" -> "0x393";
"0x394" -> "0x373" [style = dotted];
"0x373" [label = ""];
"0x373" -> "0x2b5";
"0x373" -> "0x36a" [style = dotted];
"0x288" [label = ""];
"0x288" -> "0x287" [style = dotted];
"0x32d" [label = ""];
"0x32d" -> "0x32c" [style = dotted];
"0x393" [label = ""];
"0x393" -> "0x2f8";
"0x393" -> "0x345" [style = dotted];
"0x36a" [label = ""];
"0x36a" -> "0x368";
"0x36a" -> "0x369" [style = dotted];
"0x287" [label = ""];
"0x287" -> "0x286" [style = dotted];
"0x32c" [label = ""];
"0x32c" -> "0x306";
"0x345" [label = ""];
"0x345" -> "0x332" [style = dotted];
"0x2b5" [label = ""];
"0x2b5" -> "0x293";
"0x2f8" [label = ""];
"0x2f8" -> "0x2f7" [style = dotted];
"0x368" [label = ""];
"0x368" -> "0x331";
"0x368" -> "0x2fd" [style = dotted];
"0x332" [label = ""];
"0x332" -> "0x331";
"0x293" [label = ""];
"0x293" -> "0x276";
"0x286" [label = ""];
"0x286" -> "0x285" [style = dotted];
"0x369" [label = ""];
"0x369" -> "0x31e";
"0x369" -> "0x2fd" [style = dotted];
"0x306" [label = ""];
"0x306" -> "0x285";
"0x2f7" [label = ""];
"0x2f7" -> "0x276" [style = dotted];
"0x2fd" [label = ""];
"0x2fd" -> "0x2fc";
"0x276" [label = ""];
"0x276" -> "0x23f";
"0x331" [label = ""];
"0x331" -> "0x330" [style = dotted];
"0x31e" [label = ""];
"0x31e" -> "0x2fc" [style = dotted];
"0x285" [label = ""];
"0x285" -> "0x23f" [style = dotted];
"0x2fc" [label = "0.5"];
"0x23f" [label = "1"];
"0x330" [label = "0.25"];
}


prism-rs HEAD*
direnv loaded/allowed ❯ cargo run -- --model tests/dtmc/simple2_dtmc.prism --model-type dtmc
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/prism-rs --model tests/dtmc/simple2_dtmc.prism --model-type dtmc`

            _
 _ __  _ __(_)___ _ __ ___            _ __ ___
| '_ \| '__| / __| '_ ` _ \   _____  | '__/ __|
| |_) | |  | \__ \ | | | | | |_____| | |  \__ \
| .__/|_|  |_|___/_| |_| |_|         |_|  |___/
|_|


Parsing DTMC model from file: tests/dtmc/simple2_dtmc.prism
Parsing successful
Model analysis successful:
  Module names: ["M"]
Constructed Transition ADD
Reachability (BFS): 4 iterations, reachable states: 6
Added self-loops to 0 dead-end states
Symbolic DTMC:
  Variables:
    s: curr nodes [N55b2a2398e40, N55b2a2398e80, N55b2a2398ec0], next nodes [N55b2a2398e60, N55b2a2398ea0, N55b2a2398ee0]
  Transitions ADD node ID: AddNode(N55b2a239b780)
  Transitions 0-1 ADD node ID: BddNode(N55b2a239b9e1)
  Num Nodes ADD: 26, Num Terminals: 4, Transitions(minterms): 9