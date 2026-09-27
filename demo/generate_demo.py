#!/usr/bin/env python3
"""Generate a deterministic mixed ACE demo corpus without third-party packages."""
from pathlib import Path
import random
out=Path(__file__).parent/'data';out.mkdir(parents=True,exist_ok=True)
r=random.Random(42)
with (out/'mixed-demo.bin').open('wb') as f:
    f.write(bytes([0])*2*1024*1024)
    f.write(bytes((i%256 for i in range(2*1024*1024))))
    for i in range(30_000): f.write(f'{{"ts":{1700000000+i},"status":"OK","service":"graphnet","value":{i%17}}}\n'.encode())
    f.write(bytes(r.randrange(256) for _ in range(2*1024*1024)))
print(out/'mixed-demo.bin')
