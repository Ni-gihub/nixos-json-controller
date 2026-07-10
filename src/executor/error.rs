#[derive(Debug)]
pub enum ExecutorError {

    PackageError(String),

    ServiceError(String),

}