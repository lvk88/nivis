To enable profiling, this section is needed in Cargo.toml:

```
[package.metadata.wasm-pack.profile.release]
# previously had just ['-O4']
wasm-opt = ['-O4', '-g']
```

Also, the wasm pack plugin from webpack needs to be disabled.

See:
https://github.com/rustwasm/wasm-pack/issues/797

To enable SIMD, invoke the build with:
```
RUSTFLAGS="-Ctarget-feature=+simd128" wasm-pack build --release
```
