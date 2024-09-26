const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const { ContractPromise } = require('@polkadot/api-contract');
const fs = require('fs');
const path = require('path');

const ALEPHZERO_TESTNET_WS = 'wss://testnet.alephzero.org';

async function deploy() {
  // Initialize the API
  const wsProvider = new WsProvider(ALEPHZERO_TESTNET_WS);
  const api = await ApiPromise.create({ provider: wsProvider });

  // Read the contract ABI and Wasm
  const abi = JSON.parse(fs.readFileSync(path.join(__dirname, '../contract/target/ink/event_ticket_system.json'), 'utf8'));
  const wasm = fs.readFileSync(path.join(__dirname, '../contract/target/ink/event_ticket_system.wasm'));

  // Initialize the contract
  const contract = new ContractPromise(api, abi, wasm);

  // Initialize the deployer account
  const keyring = new Keyring({ type: 'sr25519' });
  const deployer = keyring.addFromUri('//Alice'); // Replace with your actual account seed

  console.log('Deploying contract...');

  try {
    // Deploy the contract
    const gasLimit = 100000n * 1000000n;
    const storageDepositLimit = null;
    const { gasRequired, storageDeposit, result } = await contract.tx
      .new({ gasLimit, storageDepositLimit })
      .signAndSend(deployer);

    // Wait for the transaction to be included in a block
    await new Promise((resolve) => {
      contract.tx.new({ gasLimit, storageDepositLimit }).signAndSend(deployer, (result) => {
        if (result.status.isInBlock || result.status.isFinalized) {
          console.log('Contract deployed successfully!');
          console.log('Contract address:', result.contract.address.toString());
          resolve();
        }
      });
    });
  } catch (error) {
    console.error('Error deploying contract:', error);
  } finally {
    await api.disconnect();
  }
}

deploy().catch(console.error);