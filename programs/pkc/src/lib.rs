use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod pkc {

    use super::*;
    use std::convert::TryInto;

    pub fn initialize(
        ctx: Context<Initialize>,
        public_key: Vec<u8>,
        _seed: Vec<u8>,
        _bump: u8,
    ) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.public_key = public_key;
        Ok(())
    }

    pub fn commit_value(
        ctx: Context<WithAccount>,
        nonce: Vec<u8>,
        encrypted_value: Vec<u8>,
    ) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.nonce = nonce;
        acc.encrypted_value = encrypted_value;
        Ok(())
    }

    pub fn publish_secret(ctx: Context<WithAccount>, secret_key: Vec<u8>) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;
        acc.secret_key = secret_key;
        Ok(())
    }

    pub fn reveal_value(ctx: Context<WithAccount>) -> ProgramResult {
        let acc = &mut ctx.accounts.encrypted_account;

        /*
        use xsalsa20poly1305::{aead::Aead, Nonce};
        let nonce = Nonce::from_slice(acc.nonce.as_slice());

        let public_key_bytes: [u8; crypto_box::KEY_SIZE] =
            acc.public_key.as_slice().try_into().unwrap();
        let public_key = crypto_box::PublicKey::from(public_key_bytes);

        let secret_key_bytes: [u8; crypto_box::KEY_SIZE] =
            acc.secret_key.as_slice().try_into().unwrap();
        let secret_key = crypto_box::SecretKey::from(secret_key_bytes);

        let cypher = crypto_box::SalsaBox::new(&public_key, &secret_key);
        solana_program::log::sol_log_compute_units();

        let plaintext = cypher
            .decrypt(nonce, acc.encrypted_value.as_slice())
            .unwrap();

        */

        use xsalsa20poly1305::aead::{Aead, NewAead};
        use xsalsa20poly1305::{Key, Nonce, XSalsa20Poly1305};
        let nonce = Nonce::from_slice(acc.nonce.as_slice()); // 24-bytes; unique
        let key = Key::from_slice(acc.secret_key.as_slice());
        let cypher = XSalsa20Poly1305::new(key);
        let plaintext = cypher
            .decrypt(nonce, acc.encrypted_value.as_slice())
            .unwrap();

        solana_program::log::sol_log_compute_units();

        msg!(
            "plaintext={}",
            std::str::from_utf8(plaintext.as_slice()).unwrap()
        );

        acc.decrypted_value = plaintext;
        // panic!("post");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(_public_key: Vec<u8>, seed: Vec<u8>, bump: u8)]

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
    pub nonce: Vec<u8>,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub encrypted_value: Vec<u8>,
    pub decrypted_value: Vec<u8>,
}
