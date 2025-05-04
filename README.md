
# MiniPool


This is a stupid thread pool for running parallel cpu-bound tasks.

No have dependencies.

```rust
fn main() {
    let mut pool = MiniPooll::new();
    pool.spawn(|| {
        std::thread::sleep(Duration::from_secs(2))
    });

    pool.join_all();
}
```

How to use?
```rust
[dependencies]
minipool = "*"
```


## Contributing

Contributions are always welcome!


## License

[Apache](https://choosealicense.com/licenses/apache/)

