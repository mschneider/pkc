use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer, transport::TransportError};

use program_test::*;

mod program_test;

#[allow(unaligned_references)]
#[tokio::test]
async fn all() -> Result<(), TransportError> {
    let context = TestContext::new().await;

    let payer = &context.users[0].key;
   
    Ok(())
}
