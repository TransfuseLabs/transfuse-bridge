2.3. Deployment Instructions
Compile the Smart Contract:

Use the Solidity compiler or tools like Truffle or Hardhat.

```bash
npx hardhat compile
```

Deploy the Contract:

```bash
npx hardhat run scripts/deploy.js --network YOUR_NETWORK
```

Replace YOUR_NETWORK with the appropriate network (e.g., rinkeby, mainnet).

Configure the Contract:

Set the usdcToken address to the USDC contract on the Ethereum network you're using.
