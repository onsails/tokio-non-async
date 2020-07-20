use tokio::sync::mpsc::error::{SendError, TryRecvError, TrySendError};
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub trait BlockingRecv<T> {
    fn optimistic_blocking_recv(&mut self) -> Option<T>;

    fn blocking_recv(&mut self) -> Option<T>;
}

impl<T> BlockingRecv<T> for Receiver<T> {
    /// First try an optimistic `Receiver::try_recv`,
    /// the if value is unavailable, block until value is available and return it
    fn optimistic_blocking_recv(&mut self) -> Option<T> {
        match self.try_recv() {
            Ok(value) => Some(value),
            Err(TryRecvError::Empty) => self.blocking_recv(),
            Err(TryRecvError::Closed) => None,
        }
    }

    /// Blocks until value is available
    fn blocking_recv(&mut self) -> Option<T> {
        futures::executor::block_on(self.recv())
    }
}

pub trait BlockingSend<T> {
    fn optimistic_blocking_send(&mut self, message: T) -> Result<(), SendError<T>>;

    fn blocking_send(&mut self, message: T) -> Result<(), SendError<T>>;
}

impl<T> BlockingSend<T> for Sender<T> {
    fn optimistic_blocking_send(&mut self, message: T) -> Result<(), SendError<T>> {
        match self.try_send(message) {
            Ok(()) => Ok(()),
            Err(TrySendError::Closed(value)) => Err(SendError(value)),
            Err(TrySendError::Full(value)) => self.blocking_send(value),
        }
    }

    fn blocking_send(&mut self, message: T) -> Result<(), SendError<T>> {
        futures::executor::block_on(self.send(message))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test(threaded_scheduler)]
    async fn optimistic_blocking() {
        let (mut tx, mut rx) = mpsc::channel(10);

        tokio::task::spawn_blocking(move || {
            for i in 0i32..10 {
                tx.optimistic_blocking_send(i).unwrap();
            }
        })
        .await
        .unwrap();

        tokio::task::spawn_blocking(move || {
            for i in 0i32..10 {
                let received = rx.optimistic_blocking_recv();
                assert_eq!(received, Some(i));
            }

            assert_eq!(rx.optimistic_blocking_recv(), None);
        })
        .await
        .unwrap();
    }

    #[tokio::test(threaded_scheduler)]
    async fn blocking() {
        let (mut tx, mut rx) = mpsc::channel(10);

        tokio::task::spawn_blocking(move || {
            for i in 0i32..10 {
                tx.blocking_send(i).unwrap();
            }
        })
        .await
        .unwrap();

        tokio::task::spawn_blocking(move || {
            for i in 0i32..10 {
                let received = rx.blocking_recv();
                assert_eq!(received, Some(i));
            }

            assert_eq!(rx.blocking_recv(), None);
        })
        .await
        .unwrap();
    }
}
