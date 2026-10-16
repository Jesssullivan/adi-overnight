# adi-overnight

Hi friends!  Here is an ultraminimal alternate solution to the SMS simulation.  

It was written live on camera in one take, from about 4:15am to 7:20am on 16 Oct 2026
- *I see now my ffmpeg capture is uh, skewed pretty badly :eyes: apologies for the crazy artifacts and smearing*
- *clip also halted ~2hrs in I see after bluetooth earphones died, taking the whole capture with it, ahhg*
- *also, I apologies for making weird faces sometimes while programming and playing guitar while thinking*



[![Ultraminimal](https://img.youtube.com/vi/0XtlG_ln7QE/maxresdefault.jpg)](https://youtu.be/0XtlG_ln7QE)


### My cooler submission can be found here:
- docs at https://adi.clients.xoxd.ai/docs/design
- repo at https://github.com/Jesssullivan/adi-sms



## Run

```sh
cargo run --release 
cargo run --release -- 200 10 50 10 250 # [messages] [senders] [mean_ms] [fail_pct] [every_ms]
```

### RNG and determinism from two miles away

- `rand()` is a SplitMix64 stream with a single counter, seeded, stepped with a lock-free `fetch_add`. It costs about
  2 ns per draw, which I want to note is way faster than `RandomState::`. However, the senders race for the next value, so which message gets which draw depends on scheduling, and no two runs print the same numbers which makes for a poor *simulation*.  The good thing isQuickMix64 can be implemented in a fully keyed and communicative fashion, as I demonstrated and described at greatere detail in my Haskell and python 3.15 implementations.
- `RandomState` (I think its a SipHash under a random key draw from machine, though I could be wrong about that) needs no extra code, but it costs about 12 ns per draw and can never be reproduced. TinyMT32 costs about 11 ns, which is an RNG I use somewhat regularly in recreational GPU programming. 

None of these costs matter next to a 100 ms send, so the real win on a true simulation is being able to handle sleeps and folded, composable functions outside of the engineered time in flight, as the probability and outcomes can and should be the same with and without the latency.


A seed alone does not make a concurrent simulation reproducible, as demonstrated by my streaming QuickMix64 here; keying each draw by message identity does.  This means each outcome is different, and analogizing outcomes would need to be within bins over a distribution instead of purely reproducible from a seed value.



---

Cheers,
-Jess


