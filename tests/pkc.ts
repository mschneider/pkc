import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";
import { Pkc } from "../target/types/pkc";
import { expect } from "chai";
import fs from "fs";
import JSEncrypt from "node-jsencrypt";
import { accountSize } from "@project-serum/anchor/dist/cjs/coder";

describe("pkc", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Pkc as Program<Pkc>;

  const publicKey = fs.readFileSync("./tests/rsa1024-pub.pem", "utf8");
  const secretKey = fs.readFileSync("./tests/rsa1024-priv.pem", "utf8");
  const cipherText = fs.readFileSync(
    "./tests/rsa1024-ciphertext.bin",
    "binary"
  );

  it("reads the key files", async () => {
    const crypt = new JSEncrypt();
    crypt.setPublicKey(publicKey);
    expect(crypt.key.n.toString(16)).to.eq(
      "cc34188fbda3a6841e11045cb7711c8fd78037d876df69bc730983b390fdf8458630f1b406bfd082ba95481bbf960282632d51b7e44c95a28768e347d9bcaa25dca9fa2aea1b14d45c612c1ac76b357dec842f67ae887d5653f85c134b6bcf414a9c888b669ac54e6cbc85b4ab130156b1d21222565fd15a1f25a09d506708cf"
    );
    expect(crypt.key.e).to.eq(65537);

    crypt.setPrivateKey(secretKey);
    expect(crypt.key.n.toString(16).toUpperCase()).to.eq(
      "CC34188FBDA3A6841E11045CB7711C8FD78037D876DF69BC730983B390FDF8458630F1B406BFD082BA95481BBF960282632D51B7E44C95A28768E347D9BCAA25DCA9FA2AEA1B14D45C612C1AC76B357DEC842F67AE887D5653F85C134B6BCF414A9C888B669AC54E6CBC85B4AB130156B1D21222565FD15A1F25A09D506708CF"
    );
    expect(crypt.key.e).to.eq(65537);
    expect(crypt.key.d.toString(16).toUpperCase()).to.eq(
      "21515AB491479352B92923A211183685CDAE90EE13AF2E2C5E44AE256D41D2F15D0CBD53174AD2B591C5EBA7036271745EC4353220E0D2055BBCA460C3C901A5B30899D072B64397F7E0024992CC484057911769BCD712F3679AA379CE65BBB0FCD81B1FFD8BB27CA4C9041D7530D47680D97938B40BC893141FB349E6E6A4C9"
    );
    expect(crypt.key.p.toString(16).toUpperCase()).to.eq(
      "F55290302B010A805A30D2F4BC8233F4EEF3A40AA097B0068866D3950D05680BB1811B4F311C5F9DD11D0A56A4F9C846C1232C20B6B33AF4154035AA5B7B95F3"
    );
    expect(crypt.key.q.toString(16).toUpperCase()).to.eq(
      "D517602FBD317EBF46C7B2DD79C8AF425E83CEEDC5677B65FFC233FEBCD1A800078B53C280572845F7F89DC24E0ED74177663031E7F6CA171CDCD6491D8EECB5"
    );
  });

  it("decrypts", async () => {
    // const message = ;
    const crypt = new JSEncrypt();
    crypt.setPrivateKey(secretKey);
    const message = crypt.decrypt(
      Buffer.from(cipherText, "binary").toString("base64")
    );
    expect(message).to.eq("The quick brown fox jumps over the lazy dog");
  });

  it("commits and reveals", async () => {
    const publicKey = fs.readFileSync("./tests/rsa768-pub.pem", "utf8");
    const secretKey = fs.readFileSync("./tests/rsa768-priv.pem", "utf8");
    const cipherText = fs.readFileSync(
      "./tests/rsa768-ciphertext.bin",
      "binary"
    );
    const seed = Buffer.from("seed", "utf8");
    const [pda, bump] = findProgramAddressSync([seed], program.programId);

    await program.rpc.initialize(publicKey, seed, bump, {
      accounts: {
        payer: provider.wallet.publicKey,
        encryptedAccount: pda,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });

    await program.rpc.commitValue(Buffer.from(cipherText, "binary"), {
      accounts: {
        encryptedAccount: pda,
      },
    });

    await program.rpc.publishSecret(secretKey, {
      accounts: { encryptedAccount: pda },
    });

    await program.rpc.prepareDecryption({
      accounts: { encryptedAccount: pda },
    });

    await program.rpc.revealValue({
      accounts: { encryptedAccount: pda },
    });

    const acc = await program.account.encryptedAccount.fetch(pda);
    console.log("encryptedAccount", acc);
    console.log("message", acc.decryptedValue.toString());
  });
});
