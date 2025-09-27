# Dx

Enhance Developer Experience

As you can see the log is taking too much time as it as just taking 20 or 30 microseconds - so please make sure to remove this html rewrite from the time calculations + sorry please make a attr called dx that will show the whole classes use in that custome class and please add that dx attr again and only put that attr in dev - we will remove all dx attr in production - and at a time in a element only one @ sign should be present, so if there 
```bash
[dx-style] auto group nbft -> none border flex text-red-500
[dx-style] group nbft -> none border flex text-red-500
Initial: 2 added, 1 removed | (Total: 2.33ms -> Hash: 8µs, Parse: 11µs, Diff: 1µs, Cache: 115µs, Write: 2.20ms [mode=full classes=5 bytes=21720 layers+gen=159.3µs utilities=1.9052ms flush=123.2µs])
[dx-style] group card -> none border flex text-red-500
Processed: 1 added, 0 removed | (Total: 196µs -> Hash: 0µs, Parse: 7µs, Diff: 1µs, Cache: 152µs, Write: 35µs [mode=add classes=1 bytes=109 gen=25.9µs build=0ns flush=8.8µs])
[dx-style] group nbft -> none border flex text-red-500
Processed: 0 added, 1 removed | (Total: 185µs -> Hash: 0µs, Parse: 8µs, Diff: 1µs, Cache: 165µs, Write: 10µs [mode=remove classes=1 bytes=106 blank=400ns])0ns
```
