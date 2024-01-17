pub mod rpc {
    pub type RpcResult<T,E = RpcError> = Result<Option<T>, E>;
    
    #[derive(Debug)]
    pub enum RpcError {
        ConnectionError,
        TimeoutError,
        DataError,
    }
}
pub enum MyEnum<T, E = rpc::RpcError> {
    Ok(rpc::RpcResult<T,E>),
    Err(E),
}

fn main() {
    let val = MyEnum::Ok::<String>(Ok(Some("Hello".to_owned())));
    let result = val.flatten();
}
