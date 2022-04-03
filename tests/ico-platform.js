const anchor = require("@project-serum/anchor");
const assert = require("assert");

const {
    TOKEN_PROGRAM_ID,
    sleep,
    getTokenAccount,
    createMint,
    createTokenAccount,
    mintToAccount,
} = require("./utils");

const { PublicKey } = require("@solana/web3.js");

describe("ico-platform", () => {
    const provider = anchor.Provider.local();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.IdoPool;

    const nativeIcoAmount = new anchor.BN(5000000);

    let usdcMint = null;
    let nativeMint = null;
    let creatorUsdc = null;
    let creatorNative = null;

    it("Initializes the state-of-the-world", async () => {
        usdcMint = await createMint(provider);
        nativeMint = await createMint(provider);
        creatorUsdc = await createTokenAccount(
            provider,
            usdcMint,
            provider.wallet.publicKey
        );
        creatorNative = await createTokenAccount(
            provider,
            nativeMint,
            provider.wallet.publicKey
        );
        await mintToAccount(
            provider,
            nativeMint,
            creatorNative,
            nativeIcoAmount,
            provider.wallet.publicKey
        );
        creator_native_account = await getTokenAccount(provider, creatorNative);
        assert.ok(creator_native_account.amount.eq(nativeIcoAmount));
    });
});
