import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { Pkc } from "../target/types/pkc";
import nacl from "tweetnacl";

import chai, { expect } from "chai";
import chai_bytes from "chai-bytes";
chai.use(chai_bytes);

describe("pkc", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Pkc as Program<Pkc>;

  it("commits and reveals", async () => {
    const aliceKeypair = nacl.box.keyPair();
    const bobKeypair = nacl.box.keyPair();

    const nonce = nacl.randomBytes(nacl.box.nonceLength);
    const message = Buffer.from(
      "The quick brown fox jumps over the lazy dog",
      "utf-8"
    );

    // generate cyphertext with alice' secret and bob's public key
    const cipherText = nacl.box(
      message,
      nonce,
      bobKeypair.publicKey,
      aliceKeypair.secretKey
    );

    // generate decryption key with bob's secret and alice' public key
    const sharedKey = nacl.box.before(
      aliceKeypair.publicKey,
      bobKeypair.secretKey
    );

    const seed = Buffer.from("seed", "utf8");
    const [pda, bump] = findProgramAddressSync([seed], program.programId);

    await program.rpc.initialize(
      Buffer.from(aliceKeypair.publicKey),
      seed,
      bump,
      {
        accounts: {
          payer: provider.wallet.publicKey,
          encryptedAccount: pda,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      }
    );

    await program.rpc.commitValue(Buffer.from(nonce), Buffer.from(cipherText), {
      accounts: {
        encryptedAccount: pda,
      },
    });

    await program.rpc.publishSecret(Buffer.from(sharedKey), {
      accounts: { encryptedAccount: pda },
    });

    await program.rpc.revealValue({
      accounts: { encryptedAccount: pda },
    });

    const acc = await program.account.encryptedAccount.fetch(pda);
    // console.log("encryptedAccount", acc);
    expect(acc.nonce).to.equalBytes(nonce);
    expect(acc.publicKey).to.equalBytes(aliceKeypair.publicKey);
    expect(acc.secretKey).to.equalBytes(sharedKey);
    expect(acc.encryptedValue).to.equalBytes(cipherText);
    expect(acc.decryptedValue).to.equalBytes(message);
  });
});
