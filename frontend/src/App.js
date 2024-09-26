import React, { useState, useEffect } from 'react';
import { web3Accounts, web3Enable } from '@polkadot/extension-dapp';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { ContractPromise } from '@polkadot/api-contract';

import contractAbi from './contract-abi.json';

const CONTRACT_ADDRESS = 'YOUR_CONTRACT_ADDRESS_HERE';
const ALEPHZERO_TESTNET_WS = 'wss://testnet.alephzero.org';

function App() {
  const [api, setApi] = useState(null);
  const [contract, setContract] = useState(null);
  const [account, setAccount] = useState(null);
  const [userTickets, setUserTickets] = useState([]);
  const [eventName, setEventName] = useState('');
  const [expirationDate, setExpirationDate] = useState('');
  const [ticketId, setTicketId] = useState('');
  const [ticketPrice, setTicketPrice] = useState('');

  useEffect(() => {
    const initializeApi = async () => {
      const wsProvider = new WsProvider(ALEPHZERO_TESTNET_WS);
      const api = await ApiPromise.create({ provider: wsProvider });
      setApi(api);

      const contract = new ContractPromise(api, contractAbi, CONTRACT_ADDRESS);
      setContract(contract);

      await web3Enable('Event Ticket System');
      const accounts = await web3Accounts();
      if (accounts.length > 0) {
        setAccount(accounts[0]);
      }
    };

    initializeApi();
  }, []);

  const issueTicket = async () => {
    if (!contract || !account) return;

    const expirationTimestamp = Math.floor(new Date(expirationDate).getTime() / 1000);

    await contract.tx.issueTicket({ value: 0, gasLimit: -1 }, eventName, expirationTimestamp)
      .signAndSend(account.address, { signer: account.signer }, (result) => {
        if (result.status.isInBlock) {
          console.log('Transaction included in block');
          fetchUserTickets();
        }
      });
  };

  const listTicket = async () => {
    if (!contract || !account) return;

    await contract.tx.listTicket({ value: 0, gasLimit: -1 }, ticketId, ticketPrice)
      .signAndSend(account.address, { signer: account.signer }, (result) => {
        if (result.status.isInBlock) {
          console.log('Ticket listed successfully');
          fetchUserTickets();
        }
      });
  };

  const buyTicket = async () => {
    if (!contract || !account) return;

    await contract.tx.buyTicket({ value: ticketPrice, gasLimit: -1 }, ticketId)
      .signAndSend(account.address, { signer: account.signer }, (result) => {
        if (result.status.isInBlock) {
          console.log('Ticket bought successfully');
          fetchUserTickets();
        }
      });
  };

  const fetchUserTickets = async () => {
    if (!contract || !account) return;

    const { result, output } = await contract.query.getUserTickets(account.address, { value: 0, gasLimit: -1 }, account.address);
    if (result.isOk) {
      setUserTickets(output.toHuman());
    }
  };

  useEffect(() => {
    if (contract && account) {
      fetchUserTickets();
    }
  }, [contract, account]);

  return (
    <div className="App">
      <h1>Event Ticket System</h1>
      {account ? (
        <>
          <h2>Connected Account: {account.address}</h2>
          <h3>Issue Ticket</h3>
          <input
            type="text"
            placeholder="Event Name"
            value={eventName}
            onChange={(e) => setEventName(e.target.value)}
          />
          <input
            type="datetime-local"
            value={expirationDate}
            onChange={(e) => setExpirationDate(e.target.value)}
          />
          <button onClick={issueTicket}>Issue Ticket</button>

          <h3>List Ticket</h3>
          <input
            type="number"
            placeholder="Ticket ID"
            value={ticketId}
            onChange={(e) => setTicketId(e.target.value)}
          />
          <input
            type="number"
            placeholder="Price"
            value={ticketPrice}
            onChange={(e) => setTicketPrice(e.target.value)}
          />
          <button onClick={listTicket}>List Ticket</button>

          <h3>Buy Ticket</h3>
          <input
            type="number"
            placeholder="Ticket ID"
            value={ticketId}
            onChange={(e) => setTicketId(e.target.value)}
          />
          <button onClick={buyTicket}>Buy Ticket</button>

          <h3>Your Tickets</h3>
          <ul>
            {userTickets.map((ticketId) => (
              <li key={ticketId}>Ticket ID: {ticketId}</li>
            ))}
          </ul>
        </>
      ) : (
        <p>Please connect your wallet</p>
      )}
    </div>
  );
}

export default App;