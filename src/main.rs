use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tokio::try_join;

async fn fn1() -> Result<u64, AtomaError> {
    for _ in 0..10 {
        println!("fn1");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    sleep(Duration::from_secs(3)).await;
    println!("fn1-2");
    Ok(1)
}
async fn fn2() -> Result<u64, AtomaError> {
    for _ in 0..3 {
        println!("fn2");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(AtomaError { error: 2 })
}
struct Data {
    f: dyn Future<Output = Result<u64, u64>>,
}

#[derive(Debug, Error)]
pub struct AtomaError {
    pub error: u64,
}

impl Display for AtomaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AtomaError: {}", self.error)
    }
}

#[derive(Debug, Error)]
pub enum ResultOrError {
    #[error("Atoma")]
    AtomaError(#[from] AtomaError),
    #[error("JoinError {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

#[tokio::main]
async fn main() {
    let f1_task = tokio::spawn(async move { fn1().await });
    let f1_task_abort = f1_task.abort_handle();
    let f2_task = tokio::spawn(async move { fn2().await });
    let f2_task_abort = f2_task.abort_handle();
    let f1 = async {
        let x = f1_task.await;
        Ok(x??)
    };
    let f2 = async {
        let x = f2_task.await;
        x.map_err(|e| ResultOrError::JoinError(e))?
            .map_err(|e| ResultOrError::AtomaError(e))
    };
    match try_join!(f1, f2) {
        Ok((x1, x2)) => println!("All functions completed successfully"),
        Err(e) => {
            f1_task_abort.abort();
            f2_task_abort.abort();
            println!("An error occurred");
        }
    }
}
