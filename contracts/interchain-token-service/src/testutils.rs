#![cfg(any(test, feature = "testutils"))]
pub const INTERCHAIN_TOKEN_WASM_HASH: &[u8] =
    include_bytes!("../tests/testdata/interchain_token.wasm");
