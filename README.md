# nivis
## Simulation of ice crystal growth 

This program simulates the growth of ice crystals, following the algorithm described in __Ryo Kobayashi, Modeling and numerical simulations of dendritic crystal growth__.

The simulation part is written in Rust that is compiled into webassembly. Visualization and interaction is implemented in TypeScript.

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
