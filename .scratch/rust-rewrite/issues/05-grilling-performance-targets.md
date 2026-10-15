# Grilling: what 'performance' means
Type: grilling
Status: open
Blocked by:

## Question

Decoding and rendering run in libmpv, so a Rust rewrite does not speed up playback itself. Establish the measurable targets that justify 'focus on performance': cold start time, memory at idle, UI latency on seek and scrub, large playlist handling, CPU while idle or paused. Decide the baseline (current C++ build) and how each target is measured.
