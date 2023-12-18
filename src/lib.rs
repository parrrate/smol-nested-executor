use std::{
    future::{pending, Future},
    pin::Pin,
    sync::Arc,
};

use async_executor::{Executor, Task};
use async_io::block_on;
use blocking::unblock;

#[derive(Debug, Default, Clone)]
pub struct Nested {
    executor: Arc<Executor<'static>>,
}

struct NestedTask(Option<Task<()>>);

impl Future for NestedTask {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Pin::new(self.0.as_mut().unwrap()).poll(cx)
    }
}

#[async_trait::async_trait(?Send)]
impl executor_trait::Task for NestedTask {
    async fn cancel(mut self: Box<Self>) -> Option<()> {
        self.0.take().unwrap().cancel().await
    }
}

impl Drop for NestedTask {
    fn drop(&mut self) {
        if let Some(task) = self.0.take() {
            task.detach()
        }
    }
}

impl executor_trait::Executor for Nested {
    fn block_on(&self, f: Pin<Box<dyn Future<Output = ()>>>) {
        block_on(f)
    }

    fn spawn(&self, f: Pin<Box<dyn Future<Output = ()> + Send>>) -> Box<dyn executor_trait::Task> {
        Box::new(NestedTask(Some(self.executor.spawn(f))))
    }
}

#[async_trait::async_trait]
impl executor_trait::BlockingExecutor for Nested {
    async fn spawn_blocking(&self, f: Box<dyn FnOnce() + Send + 'static>) {
        unblock(f).await
    }
}

impl Nested {
    pub async fn run(self) {
        self.executor.run(pending::<()>()).await;
    }
}
