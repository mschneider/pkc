use log::*;
use std::cell::RefCell;
use std::{str::FromStr, sync::Arc, sync::RwLock};

use solana_program::{program_option::COption, program_pack::Pack};
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};

pub use solana::*;
pub mod solana;

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = solana_sdk::account::Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

struct LoggerWrapper {
    inner: env_logger::Logger,
    program_log: Arc<RwLock<Vec<String>>>,
}

impl Log for LoggerWrapper {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if record
            .target()
            .starts_with("solana_runtime::message_processor")
        {
            let msg = record.args().to_string();
            if let Some(data) = msg.strip_prefix("Program log: ") {
                self.program_log.write().unwrap().push(data.into());
            }
        }
        self.inner.log(record);
    }

    fn flush(&self) {}
}

pub struct TestContext {
    pub solana: Arc<SolanaCookie>,
}

impl TestContext {
    pub async fn new() -> Self {
        // We need to intercept logs to capture program log output
        let log_filter = "solana_rbpf=trace,\
                    solana_runtime::message_processor=debug,\
                    solana_runtime::system_instruction_processor=trace,\
                    solana_program_test=info";
        let env_logger =
            env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(log_filter))
                .format_timestamp_nanos()
                .build();
        let program_log_capture = Arc::new(RwLock::new(vec![]));
        let _ = log::set_boxed_logger(Box::new(LoggerWrapper {
            inner: env_logger,
            program_log: program_log_capture.clone(),
        }));

        let progam_id = pkc::id();

        let mut test = ProgramTest::new(
            "pkc",
            progam_id,
            processor!(pkc::entry),
        );
        // intentionally set to half the limit, to catch potential problems early
        test.set_compute_max_units(120000);

        // Setup the environment
        let mut context = test.start_with_context().await;
        let rent = context.banks_client.get_rent().await.unwrap();

        let solana = Arc::new(SolanaCookie {
            context: RefCell::new(context),
            rent,
            program_log: program_log_capture.clone(),
        });

        TestContext {
            solana: solana.clone(),
        }
    }
}
