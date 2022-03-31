Benchmarks of various alternatives for `RwLock<Arc<T>>` in read-heavy
scenarios, including arc-swap & parking lot.

## Running

```bash
cargo +nightly bench
```

## Results

@ Intel(R) Core(TM) i7-8565U CPU @ 1.80GHz (turbo off)

(substract the baseline to measure just the overhead)

```
test arcswap                    ... bench:          55 ns/iter (+/- 4)
test arcswap_full               ... bench:         482 ns/iter (+/- 94)
test baseline                   ... bench:          33 ns/iter (+/- 1)
test mutex_4                    ... bench:         736 ns/iter (+/- 421)
test mutex_unconteded           ... bench:          59 ns/iter (+/- 5)
test rwlock_fast_4              ... bench:         565 ns/iter (+/- 35)
test rwlock_fast_uncontended    ... bench:          48 ns/iter (+/- 2)
test rwlock_parking_4           ... bench:         352 ns/iter (+/- 448)
test rwlock_parking_uncontended ... bench:          55 ns/iter (+/- 0)
test rwlock_std_4               ... bench:       1,200 ns/iter (+/- 61)
test rwlock_std_uncontended     ... bench:          81 ns/iter (+/- 48)
```

## Contributing

This repo is a personal project published as-is with no intention for
maintance. Feel free to fork though!
