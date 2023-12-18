# nivis
## Simulation of ice crystal growth 

This program simulates the growth of ice crystals, following the algorithm described in _Ryo Kobayashi, Modeling and numerical simulations of dendritic crystal growth_.

The simulation part is written in Rust that is compiled into webassembly. Visualization and interaction is implemented in TypeScript.

A live demo is available here: TODO

## Running

```
git clone ...
cd www
npm run start
```

## Building for deployment

```
cd www
npx webpack build --mode production
```

## Tests

```
cargo test
```

## References

The library is an implementation of the phase field model from:

_[1] Kobayashi, Ryo. "Modeling and numerical simulations of dendritic crystal growth." Physica D: Nonlinear Phenomena 63.3-4 (1993): 410-423. _
