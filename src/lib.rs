pub mod rpc {
    pub type RpcResult<T, E = RpcError> = Result<Option<T>, E>;

    #[derive(Debug, Default)]
    pub enum RpcError {
        ConnectionError,
        TimeoutError,
        #[default]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_ok_some() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(Some("hello".to_owned())));
        assert!(matches!(val.flatten(), Ok(s) if s == "hello"));
    }

    #[test]
    fn test_flatten_ok_none_returns_default_err() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(None));
        assert!(val.flatten().is_err());
    }

    #[test]
    fn test_flatten_err_propagates() {
        let val: MyEnum<String, rpc::RpcError> = MyEnum::Err(rpc::RpcError::ConnectionError);
        assert!(matches!(val.flatten(), Err(rpc::RpcError::ConnectionError)));
    }
}

fn main() {
    let val: MyEnum<String, rpc::RpcError> = MyEnum::Ok(Ok(Some("Hello".to_owned())));
    let result = val.flatten();
    println!("{:?}", result);
}