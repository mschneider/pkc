use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod pkc {
    use rsa::{
        pkcs1::DecodeRsaPrivateKey, pkcs8::EncodePublicKey, BigUint, PaddingScheme, RsaPrivateKey,
        RsaPublicKey,
    };

    use num_bigint::{IntoBigInt, ToBigInt};
    use num_traits::sign::Signed;
    use std::borrow::Cow;

    use super::*;
    pub fn initialize(
        ctx: Context<Initialize>,
        public_key: String,
        _seed: Vec<u8>,
        _bump: u8,
    ) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.public_key = Some(public_key);
        Ok(())
    }

    pub fn commit_value(ctx: Context<WithAccount>, encrypted_value: Vec<u8>) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.encrypted_value = encrypted_value;
        Ok(())
    }

    pub fn publish_secret(ctx: Context<WithAccount>, secret_key: String) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.secret_key = Some(secret_key);

        let mut secret_key =
            RsaPrivateKey::from_pkcs1_pem(acc.secret_key.as_ref().unwrap()).unwrap();
        secret_key.precompute();

        let public_key = RsaPublicKey::from(secret_key.clone());
        let pem = public_key.to_public_key_pem(Default::default()).unwrap();
        assert_eq!(&pem, acc.public_key.as_ref().unwrap());

        acc.cached_secret_key = bincode::serialize(&secret_key).unwrap();

        Ok(())
    }

    pub fn prepare_decryption(ctx: Context<WithAccount>) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;

        Ok(())
    }

    pub fn reveal_value(ctx: Context<WithAccount>) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        let secret_key =
            bincode::deserialize::<RsaPrivateKey>(acc.cached_secret_key.as_slice()).unwrap();

        solana_program::log::sol_log_compute_units();
        msg!("pre");

        acc.decrypted_value = secret_key
            .decrypt::<u64>(PaddingScheme::new_pkcs1v15_encrypt(), &acc.encrypted_value)
            .unwrap();

        solana_program::log::sol_log_compute_units();
        panic!("end");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_public_key: String, seed: Vec<u8>, bump: u8)]

pub struct Initialize<'info> {
    pub payer: Signer<'info>,

    #[account(init,
        seeds = [&seed],
        bump = bump,
        payer = payer,
        space = 4096)]
    pub encrypted_account: Box<Account<'info, EncryptedAccount>>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]

pub struct WithAccount<'info> {
    #[account(mut)]
    pub encrypted_account: Box<Account<'info, EncryptedAccount>>,
}

#[account]
#[derive(Default)]
pub struct EncryptedAccount {
    pub public_key: Option<String>,
    pub secret_key: Option<String>,
    pub encrypted_value: Vec<u8>,
    pub decrypted_value: Vec<u8>,

    // performance optimization
    pub cached_secret_key: Vec<u8>,
}
