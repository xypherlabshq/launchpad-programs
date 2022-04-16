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

describe("ico-platform", () => {
    const provider = anchor.Provider.local();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const program = anchor.workspace.IcoPlatform;

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

    let poolSigner = null;
    let redeemableMint = null;
    let poolNative = null;
    let poolUsdc = null;
    let poolAccount = null;

    let startIcoTs = null;
    let endIcoTs = null;

    it("Initializes the ICO Pool", async () => {
        const [_poolSigner, nonce] =
            await anchor.web3.PublicKey.findProgramAddress(
                [nativeMint.toBuffer()],
                program.programId
            );
        poolSigner = _poolSigner;
        // console.log(poolSigner.toString());

        redeemableMint = await createMint(provider, poolSigner);
        poolNative = await createTokenAccount(provider, nativeMint, poolSigner);
        poolUsdc = await createTokenAccount(provider, usdcMint, poolSigner);

        poolAccount = anchor.web3.Keypair.generate();
        const nowBn = new anchor.BN(Date.now() / 1000);
        startIcoTs = nowBn.add(new anchor.BN(5));
        endIcoTs = nowBn.add(new anchor.BN(15));
        withDrawTs = nowBn.add(new anchor.BN(19));

        await program.rpc.initializePool(
            nativeIcoAmount,
            nonce,
            startIcoTs,
            endIcoTs,
            withDrawTs,
            {
                accounts: {
                    poolAccount: poolAccount.publicKey,
                    poolSigner,
                    distributionAuthority: provider.wallet.publicKey,
                    payer: provider.wallet.publicKey,
                    creatorNative,
                    redeemableMint,
                    usdcMint,
                    nativeMint,
                    poolNative,
                    poolUsdc,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [poolAccount],
            }
        );
        creator_native_account = await getTokenAccount(provider, creatorNative);
        assert.ok(creator_native_account.amount.eq(new anchor.BN(0)));
    });

    it("Modify ico time", async () => {
        console.log("PoolAccount", poolAccount.publicKey);

        await program.rpc.modifyIcoTime(
            new anchor.BN(1),
            new anchor.BN(2),
            new anchor.BN(3),
            {
                accounts: {
                    poolAccount: poolAccount.publicKey,
                    distributionAuthority: provider.wallet.publicKey,
                    payer: provider.wallet.publicKey,
                },
            }
        );
        const pool = await program.account.poolAccount.fetch(
            poolAccount.publicKey
        );
        assert.equal(pool.startIcoTs.toString(), "1");
        assert.equal(pool.endIcoTs.toString(), "2");
        assert.equal(pool.withdrawNativeTs.toString(), "3");
    });
});
