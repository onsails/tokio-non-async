# Helpers for dealing with tokio channels from non-async code in a blocking manner

```rust
let (mut tx, mut rx) = mpsc::channel(10);

for i in 0i32..10 {
    tx.send(i).await.unwrap();
}

drop(tx);

tokio::task::spawn_blocking(move || {
    while let Some(received) = rx.optimistic_blocking_recv() {
        let received = rx.optimistic_blocking_recv();
        some_blocking_op(received);
    }
})
.await
.unwrap();
```
