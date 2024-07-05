pub mod rpc {
    pub type RpcResult<T, E = RpcError> = Result<Option<T>, E>;

    #[derive(Debug)]
    pub enum RpcError {
        ConnectionError,
        TimeoutError,
        DataError,
    }
}

pub enum MyEnum<T, E = rpc::RpcError> {
    Ok(rpc::RpcResult<T, E>),
    Err(E),
}

impl<T, E: Default> MyEnum<T, E> {
    pub fn flatten(self) -> Result<T, E> {
        match self {
            MyEnum::Ok(Ok(Some(value))) => Ok(value),
            MyEnum::Ok(_) => Err(Default::default()),
            MyEnum::Err(e) => Err(e),
        }
    }
}

fn main() {
    let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(Some("Hello".to_owned())));
    let result = val.flatten();
    println!("{:?}", result);
}