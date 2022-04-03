import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { IcoPlatform } from '../target/types/ico_platform';

describe('ico-platform', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.IcoPlatform as Program<IcoPlatform>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
