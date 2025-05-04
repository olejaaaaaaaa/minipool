
# MiniPool


This is a stupid thread pool for running parallel cpu-bound tasks.

No have dependencies.

```rust
fn main() {
    let mut pool = MiniPooll::new();
    pool.spawn(|| {
        std::thread::sleep(Duration::from_secs(2));
    });

    pool.spawn_with_timeout(|| {
        // ... some cpu task
        std::thread::sleep(Duration::from_secs(60));
        // end
    }, std::time::Duration::from_secs(2));

    pool.join_all();
    // 2s elapsed
}
```

How to use?
```rust
[dependencies]
minipooll = "*"
```


## Contributing

Contributions are always welcome!


## License

[Apache](https://choosealicense.com/licenses/apache/)

