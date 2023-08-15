# Unicode Language

This library detects language support given a list of Unicode codepoints. This is primarily useful for finding out what languages a font supports.

```rust
// Input codepoints as a vector of Unicode ranges
let codepoints = vec![[65, 121]];

// Detect languages with a threshold of 0.5
let results = detect(codepoints, 0.5);

// results[0].code = "en"
// results[1].code = "nl"
// ...
```

The result is a vector of `Match` structs, with the following signature:

```rust
struct Match {
  // ISO 639-1 language code.
  code: String,
  // English name.
  name: String,
  // Name in native script.
  native: String,
  // Number of codepoints matched.
  count: u32,
  // Score (number of codepoints matched divided by the total).
  score: f64,
}
```

The language data is derived from [Adobe Font's speakeasy library](https://github.com/typekit/speakeasy). The data is extracted at build time and statically compiled as part of the library.

## License

This library is licensed under the Apache-2.0 license. Copyright 2023, [The Type Founders](https://thetypefounders.com).
