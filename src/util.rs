use jsonrpc_core::types::request::Call;

pub struct WrappedCall(Call);

impl WrappedCall {
    pub fn new(id: usize, method: &str, params: Vec<serde_json::Value>) -> Self {
        Self(web3::helpers::build_request(id, method, params))
    }

    pub fn serialize(self, worker_name: String) -> Option<Vec<u8>> {
        let mut call = serde_json::to_value(self.0).ok()?;
        if let Some(obj) = call.as_object_mut() {
            obj.insert("worker".into(), worker_name.into());
        }
        let mut res_bytes = serde_json::to_vec(&call).ok()?;
        res_bytes.push(b"\n"[0]);
        Some(res_bytes)
    }
}

pub mod calls {
    use std::ops::Div;

    use jsonrpc_core::{Failure, Output, Success, Value};
    use num::ToPrimitive;
    use num_bigint::BigUint;
    use web3::types::{BlockId, H256, H64, U64};

    #[inline(always)]
    fn unwrap_response(output: Output) -> Result<Success, Failure> {
        match output {
            Output::Success(success) => Ok(success),
            Output::Failure(failure) => Err(failure),
        }
    }

    #[derive(Debug)]
    pub struct EthGetWorkResponse {
        pub header: H256,
        pub seed: H256,
        pub target: H256,
        pub block: U64,
    }

    impl EthGetWorkResponse {
        pub fn from_rpc(output: &Value) -> Option<Self> {
            let result = output.as_array()?;
            dbg!(result[3].clone());
            Some(Self {
                header: serde_json::from_value(result[0].clone()).ok()?,
                seed: serde_json::from_value(result[1].clone()).ok()?,
                target: serde_json::from_value(result[2].clone()).ok()?,
                block: serde_json::from_value(result[3].clone()).ok()?,
            })
        }

        /// Returns the hashrate difficulty for this share
        pub fn difficulty(&self) -> u64 {
            let two_pow256: BigUint = BigUint::from(2 as u8).pow(256);
            let difficulty = two_pow256.div(BigUint::from_bytes_be(self.target.as_bytes()));
            difficulty.to_u64().unwrap()
        }
    }
}
